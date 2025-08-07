use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{arguments, load_project};
use anyhow::{bail, Context, Result};
use regex::Regex;

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

struct DeployContext {
    build_machines: Vec<String>,
    host_filter: Regex,
    ssh_config_path: String,
    project_root: PathBuf,
    output_directory: PathBuf,
}

impl DeployContext {
    fn new(
        build_machines: Vec<String>,
        host_filter: Option<&str>,
        ssh_config: PathBuf,
        project_root: PathBuf,
        link_path: Option<&Path>,
    ) -> Result<Self> {
        log::info!("Project root: {:?}", project_root);

        if let Some(host_filter) = host_filter {
            log::info!("Host filter: '{host_filter}'");
        } else {
            log::info!("Host filter: None");
        }

        let host_filter = Regex::new(host_filter.unwrap_or(".*"))
            .context("Failed to compile regex expression for host filter")?;

        let ssh_config_path = ssh_config
            .as_os_str()
            .to_str()
            .map(|s| s.to_string())
            .context("Path to SSH config could not be encoded as UTF8")?;

        let output_directory = link_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| project_root.join("result"));
        if output_directory.exists() {
            if output_directory.is_dir() {
                std::fs::remove_dir_all(&output_directory)
            } else {
                std::fs::remove_file(&output_directory)
            }
            .context("Failed to remove old result output.")?;
        };

        Ok(Self {
            build_machines,
            host_filter,
            ssh_config_path,
            project_root,
            output_directory,
        })
    }

    fn get_hosts_list(&self) -> Result<Vec<String>> {
        let mut command = Command::new("nix");
        command.args([
            "eval",
            "--raw",
            ".#nixosConfigurations",
            "--apply",
            "pkgs: builtins.concatStringsSep \" \" (builtins.attrNames pkgs)",
        ]);

        let result = command.output().context("Failed to run `nix eval`")?;
        let stderr = String::from_utf8_lossy(&result.stderr);
        if result.status.success() {
            if !result.stderr.is_empty() {
                log::warn!("`nix eval` had stderr output: {}", stderr);
            }

            let output = String::from_utf8(result.stdout)
                .context("`nix eval` output is not utf8 encoded text")?;

            let hosts = output.split_whitespace();
            Ok(hosts.map(|s| s.to_string()).collect())
        } else {
            bail!("`nix eval` returned status {}: {}", result.status, stderr);
        }
    }

    fn deploy_ssh(&self, host: &str, hostname: &str, switch: bool) -> Result<()> {
        log::info!("Deploying {host} to {hostname}");

        let mut command = Command::new("nixos-rebuild");
        command.env("NIX_SSHOPTS", format!("-F {}", self.ssh_config_path));
        command.current_dir(&self.project_root);

        // Configure builders.
        let build_machine_list = self.build_machines.join(";");
        command.arg("--builders");
        command.arg(build_machine_list);

        // Are we committing this as a boot configuration or just a test?
        if switch {
            command.arg("switch");
        } else {
            command.arg("test");
        }

        // Configure target host.
        command.arg("--flake");
        command.arg(format!(".#{host}"));

        command.arg("--target-host");
        command.arg(hostname);

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn nixos-rebuild.")?;

        let result = child
            .wait()
            .context("Failed to wait for nixos-rebuild to complete.")?;

        if !result.success() {
            bail!("`nixos-rebuild` returned non-zero output.");
        } else {
            Ok(())
        }
    }

    fn run_build(&self, host: &str, target: &str) -> Result<()> {
        log::info!("Building '{}'", host);

        let mut command = Command::new("nix");
        command.env("NIX_SSHOPTS", format!("-F {}", self.ssh_config_path));
        command.current_dir(&self.project_root);

        // Configure builders.
        let build_machine_list = self.build_machines.join(";");
        command.arg("--builders");
        command.arg(build_machine_list);

        // Configure output path.
        let output_directory = self.output_directory.join(host);

        // Our action.
        command.arg("build");

        command.arg("--out-link");
        command.arg(output_directory);

        // Specify which output to build.
        command.arg(target);

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn nix build.")?;

        let result = child
            .wait()
            .context("Failed to wait for nix-build to complete.")?;

        if !result.success() {
            bail!("`nix build` returned non-zero output.");
        } else {
            Ok(())
        }
    }

    fn run_against_hosts(
        &self,
        list_check: impl Fn(&[String]) -> Result<()>,
        mut to_run: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        let mut host_list = self
            .get_hosts_list()
            .context("Failed to get list of hosts from flake.nix")?;

        host_list.retain(move |host| self.host_filter.captures(host).is_some());

        list_check(&host_list)?;

        for host in host_list.iter() {
            to_run(host).with_context(|| format!("Error while processing host {host}"))?;
        }

        Ok(())
    }
}

fn deploy_ssh<'a>(
    build_machines: Vec<String>,
    project_root: PathBuf,
    host_filter: Option<&str>,
    ssh_config: PathBuf,
    args: arguments::SshDeploy,
) -> Result<()> {
    let context = DeployContext::new(build_machines, host_filter, ssh_config, project_root, None)
        .context("Failed to initalize build")?;

    context.run_against_hosts(
        |list| {
            if args.destination.is_some() {
                if list.len() == 1 {
                    Ok(())
                } else {
                    bail!("Host name can only be overriden when deploying to a single host. Use a host filter to limit to a single host.")
                }
            } else {
                Ok(())
            }
        },
        |host| {
            let hostname = args
                .destination
                .clone()
                .unwrap_or_else(|| format!("root@{}", host));
            context.deploy_ssh(
                host,
                &hostname,
                args.switch
            )
        },
    )?;

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

    let context = DeployContext::new(
        build_machines,
        host_filter,
        ssh_config,
        project_root,
        args.link_path.as_ref().map(|p| p.as_path()),
    )
    .context("Failed to initalize build")?;

    context.run_against_hosts(
        |_hosts| Ok(()),
        |host| {
            context.run_build(
                host,
                &format!(".#nixosConfigurations.{host}.config.system.build.raw"),
            )
        },
    )?;

    log::info!("Build successful.");

    Ok(())
}

fn build_installer(
    build_machines: Vec<String>,
    project_root: PathBuf,
    host_filter: Option<&str>,
    ssh_config: PathBuf,
    args: arguments::InstallerISO,
) -> Result<()> {
    log::info!("Building installer ISO images.");

    let context = DeployContext::new(
        build_machines,
        host_filter,
        ssh_config,
        project_root,
        args.link_path.as_ref().map(|p| p.as_path()),
    )
    .context("Failed to initalize build")?;

    context.run_against_hosts(
        |_list| Ok(()),
        |host| {
            context.run_build(
                host,
                &format!(".#nixosConfigurations.{host}.config.system.build.installer"),
            )
        },
    )?;

    log::info!("Build successful.");

    Ok(())
}

fn populate_build_mache_args(command: &mut Command, build_machines: &[String]) {
    let build_machine_list = build_machines.join(";");
    command.arg("--builders");
    command.arg(build_machine_list);
}
