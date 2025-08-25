use std::path::PathBuf;

use crate::{arguments, load_project, ProjectContext};
use anyhow::{bail, Context, Result};

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
    let context = ProjectContext::new(build_machines, host_filter, ssh_config, project_root, None)
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

    let context = ProjectContext::new(
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

    let context = ProjectContext::new(
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
