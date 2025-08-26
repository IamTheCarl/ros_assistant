use anyhow::{Context, Result};

use crate::{arguments, ProjectContext};

pub fn ssh(args: arguments::SshCommand) -> Result<()> {
    let context = ProjectContext::load_project(vec![], args.project_root, None, None)
        .context("Failed to initalize build")?;

    let host = if let Some(host) = args.host {
        host
    } else {
        context
            .select_default_host()
            .context("Failed to select default host for robot")?
    };

    context.run_ssh(host.as_str(), args.command.as_ref().map(|c| c.as_str()))
}
