use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::arguments;

pub fn ssh(args: arguments::SshCommand) -> Result<()> {
    todo!()
    // let (project_root, ssh_config) = load_project(args.project_root)?;

    // let host = if let Some(host) = args.host {
    //     host
    // } else {
    //     let mut host_configurations = HostConfig::load_project_hosts(&project_root)
    //         .context("Failed to load configuration for project hosts.")?;

    //     // If there's only one host on the robot, just assume it's that one.
    //     if host_configurations.len() == 1 {
    //         host_configurations.pop().unwrap().hostname
    //     } else {
    //         bail!(
    //             "Multiple hosts are available for this robot. Select one with the `--host` argument."
    //         );
    //     }
    // };

    // let mut command = Command::new("ssh");
    // command.arg("-F");
    // command.arg(ssh_config);
    // command.arg(host);

    // let mut child = command
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit())
    //     .stdin(Stdio::inherit())
    //     .spawn()
    //     .context("Failed to spawn ssh.")?;

    // let result = child
    //     .wait()
    //     .context("Failed to wait for ssh to complete.")?;

    // if !result.success() {
    //     log::error!("Ssh unsuccessful.");
    // } else {
    //     log::info!("Ssh successful.");
    // }

    // Ok(())
}
