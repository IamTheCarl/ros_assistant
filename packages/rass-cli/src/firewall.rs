use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::{arguments::firewall, filter_hosts, host_config::HostConfig, load_project};

pub fn firewall(args: firewall::Command) -> Result<()> {
    let (project_root, ssh_config) = load_project(args.project_root)?;
    let host_configurations = HostConfig::load_project_hosts(&project_root)
        .context("Failed to load configuration for project hosts.")?;

    let host_filter = args.hosts.as_ref().map(|s| s.as_str());
    if let Some(host_filter) = host_filter {
        log::info!("Using host filter `{host_filter}`");
    }

    let command_to_run_on_remote = match args.subcommand {
        firewall::SubCommand::Disable(_) => "systemctl stop firewall",
        firewall::SubCommand::Reset(_) => "systemctl restart firewall",
    };

    for host in filter_hosts(host_configurations.iter(), host_filter)? {
        let mut command = Command::new("ssh");
        command.arg("-F");
        command.arg(&ssh_config);
        command.arg(&host.hostname);
        command.arg(command_to_run_on_remote);

        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .spawn()
            .context("Failed to spawn ssh.")?;

        let result = child
            .wait()
            .context("Failed to wait for ssh to complete.")?;

        if !result.success() {
            log::error!("Ssh unsuccessful.");
        } else {
            log::info!("Ssh successful.");
        }
    }
    Ok(())
}
