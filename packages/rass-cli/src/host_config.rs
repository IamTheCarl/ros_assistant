use std::{collections::HashMap, fmt::Write, path::Path, process::Command};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use toposort_scc::IndexGraph;

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct HostConfig {
    pub hostname: String,
    pub ros_assistant: RosAssistantConfig,
}

impl HostConfig {
    pub fn load_project_hosts(project_root: impl AsRef<Path>) -> Result<Vec<Self>> {
        Self::load_hosts_directory(&project_root.as_ref().join("hosts"))
            .context("Failed to read hosts directory.")
    }

    pub fn determine_deployment_order(list: &[Self]) -> Result<Vec<usize>> {
        // Now we need to figure out what order they are expected to be deployed to.
        let host_indexes: HashMap<String, usize> = list
            .iter()
            .map(|host| host.hostname.clone())
            .enumerate()
            .map(|(index, host)| (host, index))
            .collect();

        let adjacency_list = list
            .iter()
            .map(|host| -> Result<Vec<usize>> {
                host.ros_assistant
                    .jump_dependency
                    .as_ref()
                    .map(|depends_on| {
                        host_indexes
                            .get(depends_on)
                            .map(|index| vec![*index])
                            .with_context(|| {
                                format!(
                            "{} depends on host {}, which is not specified in the hosts directory.",
                            host.hostname, depends_on
                        )
                            })
                    })
                    .unwrap_or(Ok(vec![]))
            })
            .collect::<Result<Vec<Vec<usize>>>>()?;

        let index_graph = IndexGraph::from_adjacency_list(&adjacency_list);

        index_graph
            .toposort_or_scc()
            .map_err(|dependency_cycles| {
                let mut error_message =
                    "Failed to determine deployment order due to cyclic reference: ".to_string();
                for cycle in dependency_cycles {
                    let mut cycle_names =
                        cycle.iter().map(|index| &list[*index].hostname).peekable();

                    while let Some(dependency_name) = cycle_names.next() {
                        if cycle_names.peek().is_some() {
                            write!(error_message, "{dependency_name}, ").ok();
                        } else {
                            writeln!(error_message, "{dependency_name}").ok();
                        }
                    }
                }

                anyhow!(error_message)
            })
            .map(|mut list| {
                list.reverse();
                list
            })
    }

    fn load_hosts_directory(path: &Path) -> Result<Vec<Self>> {
        std::fs::read_dir(path)
            .context("Failed to read directory.")?
            .map(|directory| {
                directory
                    .map(|directory| {
                        let path = directory.path();
                        Self::load(&path).with_context(|| format!("Failed to load {path:?}."))
                    })
                    .context("Failed to read directory entry.")
            })
            .flatten()
            .collect()
    }

    fn load(path: &Path) -> Result<Self> {
        let file_name = path
            .file_stem()
            .context("File does not have name.")?
            .to_str()
            .context("Path to host nix file could not be UTF8 encoded.")?;

        let path_str = path
            .to_str()
            .context("Path to host nix file could not be UTF8 encoded.")?;
        let evaluation = format!("with import {path_str} {{ config = {{}}; pkgs=<nixpkgs>; lib={{}}; }}; {{ hostname = config.networking.hostName; ros_assistant = options.ros_assistant; }}");

        let mut command = Command::new("nix-instantiate");
        command
            .args(["--eval", "--strict", "--expr", "--json"])
            .arg(evaluation);

        let output = command
            .output()
            .context("Failed to run 'nix-instantiate'.")?;

        // If there are errors, print them out.
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
            log::error!("nix-instantiate: {stderr}",);
        }

        if output.status.success() {
            let output = output.stdout;

            let host_config: HostConfig = serde_json::from_slice(&output)
                .context("Failed to decode nix-instantiate output.")?;

            if host_config.hostname != file_name {
                bail!(format!(
                    "{:?} did not match the hostname it specified: {}",
                    path, host_config.hostname
                ));
            }

            Ok(host_config)
        } else {
            bail!("`nix-instantiate` returned with an error.");
        }
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct RosAssistantConfig {
    /// The architecture of the host's CPU.
    pub arch: Architecture,

    /// Which variable will produce the disk image.
    pub image_output: String,

    /// Which device the installer ISO should write the disk image to.
    pub target_device: Option<String>,

    #[serde(default)]
    /// Override the ssh destination used to access this host.
    /// This is intended for systems where a "master" host may be in charge of relaying ssh connections to
    /// hosts on an internal network.
    pub ssh_destination: Option<String>,

    #[serde(default)]
    /// We depend on this host for our ssh connection, which means this host must be deployed to before
    /// our host.
    pub jump_dependency: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub enum Architecture {
    #[serde(rename = "aarch64")]
    Aarch64,
    #[serde(rename = "x86_64")]
    X86_64,
}

impl Architecture {
    pub fn as_nix_system_str(&self) -> &'static str {
        match self {
            Architecture::Aarch64 => "aarch64-linux",
            Architecture::X86_64 => "x86_64-linux",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_config() {
        let config = HostConfig::load(Path::new(
            "../example_projects/raspberry_pi/hosts/raspberry-pi.nix",
        ))
        .unwrap();

        assert_eq!(
            config,
            HostConfig {
                hostname: "raspberry-pi".into(),
                ros_assistant: RosAssistantConfig {
                    arch: Architecture::Aarch64,
                    image_output: "config.system.build.sdImage".into(),
                    ssh_destination: None,
                    target_device: None,
                    jump_dependency: None,
                }
            }
        );
    }

    #[test]
    fn single_deployment() {
        assert_eq!(
            HostConfig::determine_deployment_order(&[HostConfig {
                hostname: "a".into(),
                ros_assistant: RosAssistantConfig {
                    arch: Architecture::X86_64,
                    image_output: "config.system.build.raw".into(),
                    ssh_destination: None,
                    target_device: None,
                    jump_dependency: None
                }
            }])
            .unwrap(),
            vec![0]
        );
    }

    #[test]
    fn two_deployment() {
        assert_eq!(
            HostConfig::determine_deployment_order(&[
                HostConfig {
                    hostname: "a".into(),
                    ros_assistant: RosAssistantConfig {
                        arch: Architecture::X86_64,
                        image_output: "config.system.build.raw".into(),
                        ssh_destination: None,
                        target_device: None,
                        jump_dependency: None
                    }
                },
                HostConfig {
                    hostname: "b".into(),
                    ros_assistant: RosAssistantConfig {
                        arch: Architecture::X86_64,
                        image_output: "config.system.build.raw".into(),
                        ssh_destination: None,
                        target_device: None,
                        jump_dependency: Some("a".into())
                    }
                }
            ])
            .unwrap(),
            vec![0, 1]
        );
    }
}
