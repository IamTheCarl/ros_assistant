use std::{
    io,
    process::{Command, Stdio},
};

use crate::{arguments::TunnelCommand, host_config::HostConfig, load_project};
use anyhow::{bail, Context, Result};

pub(crate) fn tunnel(args: TunnelCommand) -> Result<()> {
    let (project_root, ssh_config) = load_project(args.project_root)?;

    let host = if let Some(host) = args.host {
        host
    } else {
        let mut host_configurations = HostConfig::load_project_hosts(&project_root)
            .context("Failed to load configuration for project hosts.")?;

        // If there's only one host on the robot, just assume it's that one.
        if host_configurations.len() == 1 {
            host_configurations.pop().unwrap().hostname
        } else {
            bail!(
                "Multiple hosts are available for this robot. Select one with the `--host` argument."
            );
        }
    };

    // SSH is a lot more likely to fail, so let's spawn that first to avoid wasting time spawning
    // the tunnel application.
    let mut ssh_command = Command::new("ssh");
    ssh_command.arg("-F");
    ssh_command.arg(ssh_config);
    ssh_command.arg(host);
    ssh_command.arg("dds_bridge"); // TODO this DDS bridge also needs to be renamed.

    ssh_command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped());

    let mut ssh = ssh_command.spawn().context("Failed to spawn ssh session")?;
    let mut ssh_in = ssh.stdin.take().unwrap();
    let mut ssh_out = ssh.stdout.take().unwrap();

    // TODO I want to rename that package.
    let mut tunnel_command = Command::new("dds_bridge");
    tunnel_command
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut tunnel = tunnel_command
        .spawn()
        .context("Failed to spawn ROS Tunnel")?;
    let mut tunnel_in = tunnel.stdin.take().unwrap();
    let mut tunnel_out = tunnel.stdout.take().unwrap();

    // Spawn some threads to copy the standard inputs and output between the two processes.
    let ssh_to_tunnel_thread = std::thread::spawn(move || {
        if let Err(error) = io::copy(&mut ssh_out, &mut tunnel_in) {
            log::error!("Failed to copy ssh output to local tunnel input: {error}");
        }
    });
    let tunnel_to_ssh_thread = std::thread::spawn(move || {
        if let Err(error) = io::copy(&mut tunnel_out, &mut ssh_in) {
            log::error!("Failed to copy local tunnel output to ssh input: {error}");
        }
    });

    let tunnel_result = tunnel
        .wait()
        .context("Failed to wait for tunnel command.")?;
    let ssh_result = ssh.wait().context("Failed to wait for ssh to complete.")?;

    if let Err(error) = tunnel_to_ssh_thread.join() {
        log::error!("Tunnel to ssh thread panicked: {error:?}");
    }

    if let Err(error) = ssh_to_tunnel_thread.join() {
        log::error!("Ssh to tunnel thread panicked: {error:?}");
    }

    if ssh_result.success() && tunnel_result.success() {
        log::info!("Tunnel successful.");
    } else {
        log::error!("Tunnel unsuccessful.");
    }

    Ok(())
}
