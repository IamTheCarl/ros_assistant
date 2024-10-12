use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{arguments, filter_hosts, host_config::HostConfig, load_project};
use anyhow::{Context, Result};
use tempfile::NamedTempFile;

pub fn deploy(build_machines: Vec<String>, args: arguments::Deploy) -> Result<()> {
    let (project_root, ssh_config) = load_project(args.project_root)?;

    let host_filter = args.hosts.as_ref().map(|s| s.as_str());
    if let Some(host_filter) = host_filter {
        log::info!("Using host filter `{host_filter}`");
    }

    match args.deploy_type {
        arguments::DeployType::Ssh(ssh_args) => deploy_ssh(
            build_machines,
            project_root,
            host_filter,
            ssh_config,
            ssh_args,
        )
        .context("Failed to deploy"),
        arguments::DeployType::DiskImage(disk_args) => build_disk_images(
            build_machines,
            project_root,
            host_filter,
            ssh_config,
            disk_args,
        )
        .context("Failed to build disk image"),
        arguments::DeployType::InstallerIso(iso_args) => build_installer(
            build_machines,
            project_root,
            host_filter,
            ssh_config,
            iso_args,
        )
        .context("Failed to build disk image"),
    }
}

fn deploy_ssh<'a>(
    build_machines: Vec<String>,
    project_root: PathBuf,
    host_filter: Option<&str>,
    ssh_config: PathBuf,
    args: arguments::SshDeploy,
) -> Result<()> {
    let host_configurations = HostConfig::load_project_hosts(&project_root)
        .context("Failed to load configuration for project hosts.")?;
    let deployment_order = HostConfig::determine_deployment_order(&host_configurations)
        .context("Failed to determine deployment order.")?;

    // Indexing the host configurations should never panic because the deployment was generated from the host configuration list.
    for host in filter_hosts(
        deployment_order.iter().map(|i| &host_configurations[*i]),
        host_filter,
    )? {
        // We already checked during the host config loading that this is UTF8 encoded.
        let host_file_path = project_root
            .join("hosts")
            .join(format!("{}.nix", host.hostname))
            .to_str()
            .unwrap()
            .to_string();

        log::info!("Building '{}'", host_file_path);

        let mut command = Command::new("nixos-rebuild");

        let ssh_config = ssh_config
            .as_os_str()
            .to_str()
            .context("Path to SSH config could not be encoded as UTF8")?;
        command.env("NIX_SSHOPTS", format!("-F {}", ssh_config));

        // Configure builders.
        populate_build_mache_args(&mut command, &build_machines);

        // Are we committing this as a boot configuration or just a test?
        if args.switch {
            command.arg("switch");
        } else {
            command.arg("test");
        }

        // Configure architecture.
        command.args([
            "--option",
            "system",
            host.ros_assistant.arch.as_nix_system_str(),
        ]);

        // Host config file.
        command.arg("-I");
        command.arg(format!("nixos-config={}", host_file_path));

        // Configure target host.
        command.arg("--target-host");
        command.arg(
            args.destination
                .clone()
                .unwrap_or_else(|| format!("root@{}", host.hostname)),
        );

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn nixos-rebuild.")?;

        let result = child
            .wait()
            .context("Failed to wait for nixos-rebuild to complete.")?;

        if !result.success() {
            log::error!("Deploy unsuccessful.");
        } else {
            log::info!("Deploy successful.");
        }
    }

    Ok(())
}

fn build_disk_images(
    build_machines: Vec<String>,
    project_root: PathBuf,
    host_filter: Option<&str>,
    ssh_config: PathBuf, // TODO this config may be needed to access the build machines.
    args: arguments::DiskImage,
) -> Result<()> {
    log::info!("Building boot disk images.");
    log::info!("Project root: {:?}", project_root);
    let host_configurations = HostConfig::load_project_hosts(&project_root)
        .context("Failed to load configuration for project hosts.")?;

    for host in filter_hosts(host_configurations.iter(), host_filter)? {
        // We already checked during the host config loading that this is UTF8 encoded.
        let host_file_path = project_root
            .join("hosts")
            .join(format!("{}.nix", host.hostname))
            .to_str()
            .unwrap()
            .to_string();

        log::info!("Building '{}'", host_file_path);

        let mut command = Command::new("nix-build");
        let ssh_config = ssh_config
            .as_os_str()
            .to_str()
            .context("Path to SSH config could not be encoded as UTF8")?;
        command.env("NIX_SSHOPTS", format!("-F {}", ssh_config));

        // Configure builders.
        populate_build_mache_args(&mut command, &build_machines);

        // Configure output path.
        let output_directory = args
            .link_path
            .clone()
            .unwrap_or_else(|| project_root.join("result").join(&host.hostname));
        if output_directory.exists() {
            std::fs::remove_dir_all(&output_directory)
                .context("Failed to remove old result output.")?;
        }

        command.arg("--out-link");
        command.arg(output_directory);

        // Configure for building a disk image.
        command.arg("<nixpkgs/nixos>");
        command.args([
            "--option",
            "system",
            host.ros_assistant.arch.as_nix_system_str(),
        ]);
        command.args([
            "--extra-platforms",
            host.ros_assistant.arch.as_nix_system_str(),
        ]);
        command.args(["-A", host.ros_assistant.image_output.as_str()]);

        command.arg("-I");
        command.arg(format!("nixos-config={}", host_file_path));

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn nix-build.")?;

        let result = child
            .wait()
            .context("Failed to wait for nix-build to complete.")?;

        if !result.success() {
            log::error!("Build unsuccessful.");
        } else {
            log::info!("Build successful.");
        }
    }

    Ok(())
}

fn build_installer(
    build_machines: Vec<String>,
    project_root: PathBuf,
    host_filter: Option<&str>,
    ssh_config: PathBuf,
    args: arguments::InstallerISO,
) -> Result<()> {
    let installer_iso_script = include_str!("nix_scripts/installer_iso.nix");

    log::info!("Building installer ISO images.");
    log::info!("Project root: {:?}", project_root);
    let host_configurations = HostConfig::load_project_hosts(&project_root)
        .context("Failed to load configuration for project hosts.")?;

    for host in filter_hosts(host_configurations.iter(), host_filter)? {
        // We already checked during the host config loading that this is UTF8 encoded.
        let host_file_path = project_root
            .canonicalize()
            .context("Failed to get absolute path to project root directory")?
            .join("hosts")
            .join(format!("{}.nix", host.hostname))
            .to_str()
            .unwrap()
            .to_string();

        log::info!("Building '{}'", host_file_path);

        let mut command = Command::new("nix-build");
        let ssh_config = ssh_config
            .as_os_str()
            .to_str()
            .context("Path to SSH config could not be encoded as UTF8")?;
        command.env("NIX_SSHOPTS", format!("-F {}", ssh_config));

        // Configure builders.
        populate_build_mache_args(&mut command, &build_machines);

        // Configure output path.
        let output_directory = args
            .link_path
            .clone()
            .unwrap_or_else(|| project_root.join("result").join(&host.hostname));
        if output_directory.exists() {
            std::fs::remove_dir_all(&output_directory)
                .context("Failed to remove old result output.")?;
        }

        command.arg("--out-link");
        command.arg(output_directory);

        // Configure for building ISO.
        command.arg("<nixpkgs/nixos>");

        // We can only pass our builder script as a file, so we need a tempfile.
        let mut installer_iso_build_file =
            NamedTempFile::new().context("Failed to create tempfile for ISO builder script")?;

        let target_device = host.ros_assistant.target_device.as_ref().context(
            "No installation target specified. Please set `option.ros_assistant.target_device`",
        )?;

        let install_script_content = installer_iso_script
            .replace(
                "target_arch",
                &format!("\"{}\"", host.ros_assistant.arch.as_nix_system_str()),
            )
            .replace("target_device", &format!("\"{}\"", &target_device))
            .replace("target_config", host_file_path.as_str());

        installer_iso_build_file
            .write_all(install_script_content.as_bytes())
            .context("Failed to write content to ISO builder script tempfile")?;
        let installer_iso_path = installer_iso_build_file
            .path()
            .to_str()
            .context("Path to ISO builder script tempfile is not UTF8 encodable")?;

        command.args([
            "--option",
            "system",
            host.ros_assistant.arch.as_nix_system_str(),
        ]);
        command.args(["-A", "config.system.build.isoImage"]);
        command.arg("-I");
        command.arg(format!("nixos-config={}", installer_iso_path));

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn nix-build.")?;

        let result = child
            .wait()
            .context("Failed to wait for nix-build to complete.")?;

        if !result.success() {
            log::error!("Build unsuccessful.");
        } else {
            log::info!("Build successful.");
        }
    }

    Ok(())
}

fn populate_build_mache_args(command: &mut Command, build_machines: &[String]) {
    let build_machine_list = build_machines.join(";");
    command.arg("--builders");
    command.arg(build_machine_list);
}
