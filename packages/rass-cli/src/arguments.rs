use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Manages robot workspaces, deployment, and integration with Home Assistant.
pub struct RosAssistant {
    #[argh(option, short = 'b')]
    /// specify a remote build machine to be used to build your project. This is especially useful for cross compiling.
    /// specify each machine as `--build-machine 'ssh://hostname x86_64-linux aarch64-linux'`, adjusting the hostname
    /// and supported architectures as needed.
    pub build_machine: Vec<String>,

    #[argh(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum SubCommand {
    NewProject(NewProject),
    Deploy(Deploy),
    Ssh(SshCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Create a new robot project.
#[argh(subcommand, name = "new")]
pub struct NewProject {}

#[derive(FromArgs, PartialEq, Debug)]
/// Build and deploy a project.
#[argh(subcommand, name = "deploy")]
pub struct Deploy {
    #[argh(option)]
    /// restrict which hosts are deployed using a regex expression
    pub hosts: Option<String>,

    #[argh(option)]
    /// specify a directory to be used as the project root (defaults to the current directory)
    pub project_root: Option<PathBuf>,

    #[argh(subcommand)]
    pub deploy_type: DeployType,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum DeployType {
    Ssh(SshDeploy),
    DiskImage(DiskImage),
    InstallerIso(InstallerISO),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Build and deploy a project over ssh.
#[argh(subcommand, name = "ssh")]
pub struct SshDeploy {
    #[argh(switch)]
    /// makes the configuration the new boot default
    pub switch: bool,

    #[argh(option)]
    /// override the default ssh destination
    pub destination: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Build a project and create an initaial boot disk image for it.
#[argh(subcommand, name = "disk")]
pub struct DiskImage {
    #[argh(option)]
    /// override the default link path for the project
    pub link_path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Build an ISO image for performing unattended installations of the disk image.
/// This image can be written to a USB drive or burned to a CD/DVD. Note that this
/// image is DESTRUCTIVE to any machine it is deployed on, as it will overwrite any
/// content on the target hard drive.
#[argh(subcommand, name = "installer")]
pub struct InstallerISO {
    #[argh(option)]
    /// override the default link path for the project
    pub link_path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Ssh into your robot's computer.
#[argh(subcommand, name = "ssh")]
pub struct SshCommand {
    #[argh(option)]
    /// specify a directory to be used as the project root (defaults to the current directory)
    pub project_root: Option<PathBuf>,

    #[argh(positional)]
    pub host: Option<String>,
}