use std::{
    fmt::Write,
    net::IpAddr,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use dns_lookup::lookup_host;
use local_ip_address::list_afinet_netifas;

use crate::arguments::firewall;

pub fn firewall(args: firewall::Command) -> Result<()> {
    todo!()
    // let (project_root, ssh_config) = load_project(args.project_root)?;
    // let host_configurations = HostConfig::load_project_hosts(&project_root)
    //     .context("Failed to load configuration for project hosts.")?;

    // let host_filter = args.hosts.as_ref().map(|s| s.as_str());
    // if let Some(host_filter) = host_filter {
    //     log::info!("Using host filter `{host_filter}`");
    // }

    // for host in filter_hosts(host_configurations.iter(), host_filter)? {
    //     let mut command = Command::new("ssh");
    //     command.arg("-F");
    //     command.arg(&ssh_config);
    //     command.arg(&host.hostname);

    //     match &args.subcommand {
    //         firewall::SubCommand::Disable(_) => {
    //             command.arg("systemctl stop firewall");
    //         }
    //         firewall::SubCommand::Reset(_) => {
    //             command.arg("systemctl restart firewall");
    //         }
    //         firewall::SubCommand::Pierce(pierce) => {
    //             let ip_table_command = generate_pierce_commands(pierce)
    //                 .context("Failed to generate command to run on remote system")?;
    //             command.arg(ip_table_command);
    //         }
    //     }

    //     let mut child = command
    //         .stdout(Stdio::inherit())
    //         .stderr(Stdio::inherit())
    //         .stdin(Stdio::inherit())
    //         .spawn()
    //         .context("Failed to spawn ssh.")?;

    //     let result = child
    //         .wait()
    //         .context("Failed to wait for ssh to complete.")?;

    //     if !result.success() {
    //         log::error!("Operation unsuccessful.");
    //     } else {
    //         log::info!("Operation successful.");
    //     }
    // }
    // Ok(())
}

fn generate_pierce_commands(command: &firewall::Pierce) -> Result<String> {
    let addresses: Vec<IpAddr> = if command.host.is_empty() {
        // Assume the current host.

        let network_interfaces =
            list_afinet_netifas().context("Failed to fetch local network interfaces")?;

        network_interfaces
            .into_iter()
            .map(|(_name, ip)| ip)
            .collect()
    } else {
        let mut all_addresses = Vec::new();
        for host in command.host.iter() {
            // The IP addresses have been provided to us.
            let mut host_addresses =
                lookup_host(host).with_context(|| format!("Failed to look up host: {host}"))?;

            all_addresses.append(&mut host_addresses);
        }

        all_addresses
    };

    // Now we build the command to run on the remote device.
    let mut command = String::new();

    for ip in addresses {
        // We do not add loopback devices to the list.
        if !ip.is_loopback() {
            let iptables = match ip {
                IpAddr::V4(_) => "iptables",
                IpAddr::V6(_) => "ip6tables",
            };

            // This write should never fail.
            writeln!(
                &mut command,
                "{iptables} -I nixos-fw -s {ip} -j nixos-fw-accept"
            )
            .unwrap();
        }
    }

    log::debug!("Run on remote:\n{command}");
    Ok(command)
}
