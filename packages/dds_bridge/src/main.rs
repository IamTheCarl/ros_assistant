use anyhow::{bail, Context, Result};
use futures::StreamExt;
use rustdds::{DomainParticipantBuilder, StatusEvented};
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> Result<()> {
    let domain = match std::env::var("ROS_DOMAIN_ID") {
        Ok(domain) => domain
            .parse::<u16>()
            .context("Failed to parse ROS_DOMAIN_ID as a 16bit unsigned integer")?,
        Err(error) => match error {
            std::env::VarError::NotPresent => 0, // Use the default.
            std::env::VarError::NotUnicode(_) => bail!("ROS_DOMAIN_ID is not unicode encoded"),
        },
    };

    let domain_participant = DomainParticipantBuilder::new(domain)
        .build()
        .context("Failed to build DDS participant")?;

    let status_listener = domain_participant.status_listener();
    let mut status_stream = status_listener.as_async_status_stream();

    let mut sig_terminate =
        signal(SignalKind::terminate()).context("Failed to hook into terminate signal.")?;
    let mut sig_interrupt =
        signal(SignalKind::interrupt()).context("Failed to hook into interrupt signal.")?;

    loop {
        tokio::select! {
            _ = sig_terminate.recv() => {
                break;
            }
            _ = sig_interrupt.recv() => {
                break;
            }
            status_update = status_stream.next() => {
                dbg!(status_update);
            }
        }
    }

    Ok(())
}
