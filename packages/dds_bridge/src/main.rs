use anyhow::{bail, Context, Result};
use bytes::Bytes;
use futures::StreamExt;
use rustdds::{
    policy::Reliability, qos::HasQoSPolicy, DomainParticipant, DomainParticipantBuilder,
    DomainParticipantStatusEvent, Keyed, QosPolicies, QosPolicyBuilder, StatusEvented, Subscriber,
    TopicKind,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::IsTerminal,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    signal::unix::{signal, SignalKind},
    sync::mpsc,
};

const TOPIC_LOOKUP_TIMEOUT_SECONDS: u64 = 15;

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();

    let domain = match std::env::var("ROS_DOMAIN_ID") {
        Ok(domain) => domain
            .parse::<u16>()
            .context("Failed to parse ROS_DOMAIN_ID as a 16bit unsigned integer")?,
        Err(error) => match error {
            std::env::VarError::NotPresent => 0, // Use the default.
            std::env::VarError::NotUnicode(_) => bail!("ROS_DOMAIN_ID is not unicode encoded"),
        },
    };

    let mut output = io::stdout();
    let mut input = io::stdin();

    if std::io::stdout().is_terminal() {
        bail!("Refusing to output binary data to terminal");
    }

    start_communications(&mut output, &mut input)
        .await
        .context("Failed to start communications")?;

    bridge_dds(domain, &mut output, input)
        .await
        .context("Failed to bridge DDS")?;

    // Tell the remote we are shutting down.
    let mut buffer = Vec::new();
    output
        .send_message(&mut buffer, &Command::Hangup)
        .await
        .ok();

    Ok(())
}

macro_rules! fatal_hangup {
    ($output:ident, $buffer:expr, $message:literal) => {
        // Tell the remote we've had an unrecoverable error.
        // This may fail to send. There's nothing we can do about that so the error is ignored.
        $output
            .send_message(
                $buffer,
                &Command::FatalError {
                    message: $message.into(),
                },
            )
            .await
            .ok();

        bail!($message);
    };
}

async fn start_communications(
    mut output: impl AsyncWriteExt + Unpin,
    mut input: impl AsyncReadExt + Unpin,
) -> Result<()> {
    let mut buffer = Vec::new();

    output
        .send_message(&mut buffer, &Header { version: 0 })
        .await
        .context("Failed to send protocol version")?;

    let dont_forget_to_uncomment_this = 0;
    // let remote_header: Header = input
    //     .recv_message(&mut buffer)
    //     .await
    //     .context("Failed to receive remote version")?;

    // if remote_header.version != 0 {
    //     fatal_hangup!(output, &mut buffer, "Remote version incompatible.");
    // }

    Ok(())
}

async fn bridge_dds(
    domain: u16,
    mut output: impl AsyncWriteExt + Unpin,
    mut input: impl AsyncReadExt + Unpin + Send + 'static,
) -> Result<()> {
    let domain_participant = Arc::new(
        DomainParticipantBuilder::new(domain)
            .build()
            .context("Failed to build DDS participant")?,
    );

    let status_listener = domain_participant.status_listener();
    let mut status_stream = status_listener.as_async_status_stream();

    let mut sig_terminate =
        signal(SignalKind::terminate()).context("Failed to hook into terminate signal.")?;
    let mut sig_interrupt =
        signal(SignalKind::interrupt()).context("Failed to hook into interrupt signal.")?;

    let mut input_buffer = Vec::new();

    let (message_tx, mut message_rx) = mpsc::channel(10);
    let message_reception = tokio::spawn(async move {
        loop {
            let message = input.recv_message::<Command>(&mut input_buffer).await;
            if let Err(error) = message_tx.send(message).await {
                log::error!("Reception error: {error}");
                break;
            }
        }
    });

    let qos = QosPolicyBuilder::new()
        .reliability(Reliability::BestEffort)
        .build();
    let subscriber = domain_participant.create_subscriber(&qos).unwrap();

    let mut output_buffer = Vec::new();

    let mut publishing_to_local_topics = HashMap::new();
    // let mut local_topic_subscriptions = HashMap::new();

    let (new_subscription_tx, new_subscription_rx) = mpsc::channel(10);

    loop {
        tokio::select! {
        _ = sig_terminate.recv() => {
            break;
        }
        _ = sig_interrupt.recv() => {
            break;
        }
        command = message_rx.recv() => {
            let command = command.context("Message queue closed")?.context("Failed to receive message")?;
            match command {
                Command::FatalError { message } => bail!("Fatal error from remote: {}", message),
                Command::Hangup => break,
                Command::DetectedTopic { name, type_name, qos, topic_kind } => {
                    match domain_participant.create_topic(name.clone(), type_name, &qos, topic_kind.into()) {
                        Ok(topic) => { publishing_to_local_topics.insert(name, topic); },
                        Err(error) => log::error!("Failed to create topic: {error}"),
                    }
                },
                Command::LostTopic { name } => {
                    publishing_to_local_topics.remove(&name);
                },
            }
        }
        status_update = status_stream.next() => {
            let status_update = status_update.unwrap();
            match status_update {
                DomainParticipantStatusEvent::TopicDetected { name, .. } => {
                    // Looking up a topic is a blocking operation.
                    let domain_participant = domain_participant.clone();
                    let new_subscription_tx = new_subscription_tx.clone();
                    let name = name.clone();

                    tokio::task::spawn_blocking(|| lookup_topic(domain_participant, new_subscription_tx, name));


                    // local_topic_subscriptions.insert(topic.topic_name().clone(), todo!());

                    // let qos = topic.topic_data.qos();
                    // output.send_message(&mut output_buffer, &Command::DetectedTopic {
                    //     name: topic.topic_data.name,
                    //     type_name: topic.topic_data.type_name,
                    //     qos,
                    //     topic_kind: if topic.topic_data.key.is_some() {
                    //         SerializableTopicKind::WithKey
                    //     } else {
                    //         SerializableTopicKind::NoKey
                    //     }
                    // }).await?;
                }
                DomainParticipantStatusEvent::TopicLost { name } => {
                    output.send_message(&mut output_buffer, &Command::LostTopic { name }).await?;
                }
                _ => {
                    // We don't care about anything else.
                }
            }
            }
        }
    }

    // We're done with that.
    message_reception.abort();

    Ok(())
}

fn lookup_topic(
    domain_participant: Arc<DomainParticipant>,
    sender: mpsc::Sender<Subscriber>,
    name: String,
) {
    fn trampoline(
        domain_participant: Arc<DomainParticipant>,
        sender: mpsc::Sender<Subscriber>,
        name: &str,
    ) -> Result<()> {
        let topic = domain_participant
            .find_topic(
                name.as_ref(),
                Duration::from_secs(TOPIC_LOOKUP_TIMEOUT_SECONDS),
            )
            .context("Failed to create topic object after finding the topic")?
            .context("Topic is not available")?;

        log::debug!("FOUND TOPIC: {topic:?}");
        match topic.kind() {
            TopicKind::NoKey => todo!(),
            TopicKind::WithKey => {
                // ROS2 does not support keyed topics (as of writing this)
                // We'll just ignore this.
            }
        }

        Ok(())
    }

    if let Err(error) = trampoline(domain_participant, sender, name.as_ref()) {
        log::error!("Failed to look up topic `{name}`: {error:?}");
    }
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: usize,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    // Something went wrong and we cannot recover. Communications are expected to terminate after this.
    FatalError {
        message: String,
    },

    // Remote is closing, and there is nothing unexpected about that.
    Hangup,

    // A new DDS topic has appeared.
    DetectedTopic {
        name: String,
        type_name: String,
        qos: QosPolicies,
        topic_kind: SerializableTopicKind,
    },

    // A DDS topic has disappeared.
    LostTopic {
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
enum SerializableTopicKind {
    NoKey,
    WithKey,
}

impl From<TopicKind> for SerializableTopicKind {
    fn from(value: TopicKind) -> Self {
        match value {
            TopicKind::NoKey => Self::NoKey,
            TopicKind::WithKey => Self::WithKey,
        }
    }
}

impl From<SerializableTopicKind> for TopicKind {
    fn from(value: SerializableTopicKind) -> Self {
        match value {
            SerializableTopicKind::NoKey => Self::NoKey,
            SerializableTopicKind::WithKey => Self::WithKey,
        }
    }
}

type LengthIndicator = u32;

trait SendMessage {
    async fn send_message(&mut self, buffer: &mut Vec<u8>, message: &impl Serialize) -> Result<()>;
}

impl<W> SendMessage for W
where
    W: AsyncWriteExt + Unpin,
{
    async fn send_message(&mut self, buffer: &mut Vec<u8>, message: &impl Serialize) -> Result<()> {
        buffer.clear();

        // Create a spacer for the length indicator.
        let length: LengthIndicator = 0;
        buffer.extend(length.to_le_bytes());

        // Write the payload to the buffer.
        bincode::serialize_into(&mut *buffer, message).context("Failed to serialize message")?;

        // Now we know the actual length of the message.
        let length: u32 =
            (buffer.len() - std::mem::size_of::<LengthIndicator>()) as LengthIndicator;
        buffer[0..std::mem::size_of::<LengthIndicator>()].copy_from_slice(&length.to_le_bytes());

        // We can finally send the message.
        self.write_all(buffer.as_slice())
            .await
            .context("Failed to send message")?;

        Ok(())
    }
}

trait RecvMessage {
    async fn recv_message<M>(&mut self, buffer: &mut Vec<u8>) -> Result<M>
    where
        M: DeserializeOwned;
}

impl<R> RecvMessage for R
where
    R: AsyncReadExt + Unpin,
{
    async fn recv_message<M>(&mut self, buffer: &mut Vec<u8>) -> Result<M>
    where
        M: DeserializeOwned,
    {
        let mut length = [0xA5; std::mem::size_of::<LengthIndicator>()];
        self.read_exact(&mut length)
            .await
            .context("Failed to read length")?;

        let length = LengthIndicator::from_le_bytes(length) as usize;
        buffer.clear();
        buffer.resize(length, 0xA5);

        self.read_exact(buffer)
            .await
            .context("Failed to read message")?;

        let message = bincode::deserialize(&buffer).context("Failed to deserialize message")?;

        Ok(message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn message_framing() {
        // Testing up to a 4kb payload.
        const NUM_STEPS: usize = 16;
        const STEP_LENGTH: usize = 0x256;

        let (mut input, mut output) = tokio::io::duplex(NUM_STEPS * STEP_LENGTH);

        let mut buffer = Vec::new();

        for i in 0..NUM_STEPS {
            let length = i * STEP_LENGTH;
            let payload: Vec<u8> = (0..length).map(|b| (b % 0xFF) as u8).collect();

            input.send_message(&mut buffer, &payload).await.unwrap();

            let result: Vec<u8> = output.recv_message(&mut buffer).await.unwrap();

            assert_eq!(payload, result);
        }
    }
}
