use std::{fmt::Write, net::IpAddr};

use anyhow::{Context, Result};
use dns_lookup::lookup_host;
use local_ip_address::list_afinet_netifas;

use crate::{arguments::firewall, ProjectContext};

pub fn firewall(args: firewall::Command) -> Result<()> {
    let host_filter = args.hosts.as_ref().map(|s| s.as_str());
    if let Some(host_filter) = host_filter {
        log::info!("Using host filter `{host_filter}`");
    }

    let context = ProjectContext::load_project(vec![], args.project_root, host_filter, None)
        .context("Failed to initalize build")?;

    let command = match &args.subcommand {
        firewall::SubCommand::Disable(_) => "systemctl stop firewall".to_string(),
        firewall::SubCommand::Reset(_) => "systemctl restart firewall".to_string(),
        firewall::SubCommand::Pierce(pierce) => generate_pierce_commands(pierce)
            .context("Failed to generate command to run on remote system")?,
    };

    context.run_against_hosts(
        |_| Ok(()),
        |host| context.run_ssh(host, Some(command.as_str())),
    )?;

    log::info!("Request completed successfully");
    Ok(())
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
