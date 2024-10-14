use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::{firewall::firewall, host_config::HostConfig};

mod arguments;
mod deploy;
mod firewall;
mod host_config;
mod ssh;

fn main() {
    let args = argh::from_env();

    colog::init();

    if let Err(error) = application(args) {
        log::error!("Fatal error: {:?}", error);
    }
}

fn application(args: arguments::RosAssistant) -> Result<()> {
    log::info!("ROS Assistant CLI v{}", std::env!("CARGO_PKG_VERSION"));

    match args.subcommand {
        arguments::SubCommand::NewProject(new_project_args) => {
            new_project(new_project_args).context("Failed to create new project")
        }
        arguments::SubCommand::Deploy(deploy_args) => {
            deploy::deploy(args.build_machine, deploy_args).context("Failed to deploy project")
        }
        arguments::SubCommand::Ssh(ssh_args) => ssh::ssh(ssh_args).context("Failed to ssh to host"),
        arguments::SubCommand::Firewall(firewall_args) => firewall(firewall_args),
    }
}

fn new_project(_args: arguments::NewProject) -> Result<()> {
    bail!("New project sub-command is not yet implemented.")
}

fn filter_hosts<'a>(
    hosts: impl Iterator<Item = &'a HostConfig>,
    filter: Option<&'a str>,
) -> Result<impl Iterator<Item = &'a HostConfig>> {
    let regex = Regex::new(filter.unwrap_or(".*"))
        .context("Failed to compile regex expression for host filter")?;

    Ok(hosts.filter(move |host| regex.captures(&host.hostname).is_some()))
}

fn load_project(project_root: Option<PathBuf>) -> Result<(PathBuf, PathBuf)> {
    let project_root = project_root
        .map(Ok)
        .unwrap_or_else(|| std::env::current_dir().context("Failed to get current directory"))?;

    log::info!("Project root: {:?}", project_root);

    let ssh_config = project_root.join("ssh_config");
    if !ssh_config.exists() {
        bail!("Project is missing `ssh_config` file.");
    }

    Ok((project_root, ssh_config))
}
