use anyhow::{bail, Context, Result};
use futures::Stream;
use hashbrown::{HashMap, HashSet};
use r2r::{Node, PublisherUntyped, QosProfile};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::IsTerminal, time::Duration};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Stdin, Stdout},
    signal::unix::{signal, SignalKind},
    sync::mpsc,
    time::MissedTickBehavior,
};

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();

    let mut output = io::stdout();
    let mut input = io::stdin();

    if std::io::stdout().is_terminal() {
        bail!("Refusing to output binary data to terminal");
    }

    log::info!("Starting bridge.");

    start_communications(&mut output, &mut input)
        .await
        .context("Failed to start communications")?;

    tunnel_ros(&mut output, input)
        .await
        .context("Failed to bridge DDS")?;

    // Tell the remote we are shutting down.
    let mut buffer = Vec::new();
    output
        .send_message(&mut buffer, &Message::Hangup)
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
                &Message::FatalError {
                    message: $message.into(),
                },
            )
            .await
            .ok();

        bail!($message);
    };
}

async fn start_communications(output: &mut Stdout, input: &mut Stdin) -> Result<()> {
    let mut buffer = Vec::new();
    const VERSION: u16 = 0;

    output
        .send_message(&mut buffer, &Header { version: VERSION })
        .await
        .context("Failed to send protocol version")?;

    let remote_header: Header = input
        .recv_message(&mut buffer)
        .await
        .context("Failed to receive remote version")?;

    if remote_header.version != VERSION {
        fatal_hangup!(output, &mut buffer, "Remote version incompatible.");
    }

    log::info!("Handshake with remote peer complete.");

    Ok(())
}

async fn tunnel_ros(output: &mut Stdout, mut input: Stdin) -> Result<()> {
    let ctx = r2r::Context::create().context("Failed to create ROS context")?;
    let mut node =
        r2r::Node::create(ctx, "ros_tunnel", "ros_tunnel").context("Failed to create ROS node")?;

    let mut update_interval = tokio::time::interval(Duration::from_secs(10));
    update_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    update_interval.reset_immediately(); // We want to update as soon as we're ready.

    let mut sig_terminate =
        signal(SignalKind::terminate()).context("Failed to hook into terminate signal.")?;
    let mut sig_interrupt =
        signal(SignalKind::interrupt()).context("Failed to hook into interrupt signal.")?;

    let mut output_buffer = Vec::new();
    let mut input_buffer = Vec::new();

    let (message_tx, mut message_rx) = mpsc::channel(10);
    let message_reception = tokio::spawn(async move {
        loop {
            let message = input.recv_message::<Message>(&mut input_buffer).await;
            if let Err(error) = message_tx.send(message).await {
                log::error!("Reception error: {error}");
                break;
            }
        }
    });

    let mut known_topics = HashMap::new();
    let mut subscribers = HashMap::new();
    let mut publishers = HashMap::new();
    let mut currently_subscribed = HashSet::new();

    loop {
        tokio::select! {
            _ = sig_terminate.recv() => {
                break;
            }
            _ = sig_interrupt.recv() => {
                break;
            }
            message = message_rx.recv() => {
                let message = message.context("Message queue closed")?.context("Failed to receive message")?;
                let control_flow = process_command(
                        &mut node,
                        &known_topics,
                        &mut subscribers,
                        &mut publishers,
                        message)
                    .context("Failed to process message from remote")?;

                match control_flow {
                    ControlFlow::Continue => continue,
                    ControlFlow::Exit => break,
                }
            }
            _ = update_interval.tick() => {
                scan_for_topics(&mut node, output, &mut output_buffer, &mut known_topics).await?;
                scan_for_subscriptions(output, &mut output_buffer, &publishers, &mut currently_subscribed).await?;
            }
        }
    }

    // We're done with that.
    message_reception.abort();

    Ok(())
}

enum ControlFlow {
    Continue,
    Exit,
}

fn process_command(
    node: &mut Node,
    known_topics: &HashMap<String, String>,
    subscribers: &mut HashMap<String, Box<dyn Stream<Item = Vec<u8>> + Unpin>>,
    publishers: &mut HashMap<String, PublisherUntyped>,
    message: Message,
) -> Result<ControlFlow> {
    match message {
        Message::FatalError { message } => bail!("Fatal error from remote: {message}"),
        Message::Hangup => return Ok(ControlFlow::Exit),
        Message::NewTopic { name, message_type } => {
            if !publishers.contains_key(&name) {
                let publisher = node
                    .create_publisher_untyped(&name, &message_type, QosProfile::default())
                    .context("Failed to create publisher")?;
                publishers.insert(name, publisher);
            } else {
                log::warn!("Remote has informed us of topic `{name}` being created twice.");
            }
        }
        Message::SetTopicSubscribed { name, subscribed } => {
            if subscribed {
                // Remote wants to subscribe, which means we need to subscribe.
                if let Some(topic_type) = known_topics.get(&name) {
                    match node.subscribe_raw(&name, topic_type, QosProfile::default()) {
                        Ok(subscription) => {
                            subscribers.insert(name, Box::new(subscription));
                        }
                        Err(error) => {
                            log::error!("Failed to subscribe to topic `{name}`: {error:?}");
                        }
                    }
                } else {
                    log::error!("Remote wanted to subscribe to unknown topic `{name}`");
                }
            } else {
                // Remote wants to unsubscribe, which means we need to unsubscribe.
                if subscribers.remove(&name).is_none() {
                    log::error!("Remote wanted to unsubscribe from a topic `{name}`, but we were never subscribed to that.");
                }
            }
        }
        Message::DeletedTopic { name } => {
            if publishers.remove(&name).is_none() {
                log::warn!("Remote has informed us of the removal of topic `{name}`, but we were never aware of such a topic.");
            }
        }
    }

    Ok(ControlFlow::Continue)
}

async fn scan_for_topics(
    node: &mut Node,
    output: &mut Stdout,
    buffer: &mut Vec<u8>,
    known_topics: &mut HashMap<String, String>,
) -> Result<()> {
    match node.get_topic_names_and_types() {
        Ok(topics) => {
            log::info!("Topics: {topics:?}");

            // Remove dead topics.
            for (name, _message_type) in
                known_topics.extract_if(|name, _message_type| !topics.contains_key(name))
            {
                // This loop gets all of the entries that were removed.
                output
                    .send_message(buffer, &Message::DeletedTopic { name })
                    .await
                    .context("Failed to send message removal notice.")?;
            }

            // Look for new topics.
            for (name, mut types) in topics {
                if !known_topics.contains_key(&name) {
                    // This is a new topic.
                    // See if we have type info. We can only subscribe to one type, so we will only inform our remote peer of
                    // the first type we see.
                    if let Some(message_type) = types.pop() {
                        known_topics.insert(name.clone(), message_type.clone());
                    } else {
                        log::error!("Topic {name} does not have a type.");
                    }
                }
            }
        }
        Err(error) => log::error!("Failed to poll for new topics: {error}"),
    }

    Ok(())
}

async fn scan_for_subscriptions(
    output: &mut Stdout,
    buffer: &mut Vec<u8>,

    publishers: &HashMap<String, PublisherUntyped>,
    currently_subscribed: &mut HashSet<String>,
) -> Result<()> {
    for (name, publisher) in publishers.iter() {
        match publisher.get_inter_process_subscription_count() {
            Ok(inter_process_subscription_count) => {
                let is_subscribed_to = inter_process_subscription_count > 0;
                let was_subscribed_to = currently_subscribed.contains(name);

                match (is_subscribed_to, was_subscribed_to) {
                    (true, false) => {
                        // We need to subscribe to this topic.
                        output
                            .send_message(
                                buffer,
                                &Message::SetTopicSubscribed {
                                    name: name.clone(),
                                    subscribed: true,
                                },
                            )
                            .await
                            .context("Failed to send subscription request.")?;

                        currently_subscribed.insert(name.clone());
                    }
                    (false, true) => {
                        // We need to unsubscribe from this topic.
                        output
                            .send_message(
                                buffer,
                                &Message::SetTopicSubscribed {
                                    name: name.clone(),
                                    subscribed: false,
                                },
                            )
                            .await
                            .context("Failed to send unsubscribe request.")?;

                        currently_subscribed.remove(name);
                    }
                    _ => {
                        // Nothing needs to be done here.
                    }
                }
            }
            Err(error) => {
                log::error!(
                    "Failed to get inter process subscription count for topic `{name}`: {error:?}"
                )
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: u16,
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    // Something went wrong and we cannot recover. Communications are expected to terminate after this.
    FatalError { message: String },

    // Remote is closing, and there is nothing unexpected about that.
    Hangup,

    // A new topic has appeared.
    NewTopic { name: String, message_type: String },

    // Inform the remote that a topic has been subscribed to.
    SetTopicSubscribed { name: String, subscribed: bool },

    // A topic has disappeared.
    DeletedTopic { name: String },
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

        self.flush()
            .await
            .context("Failed to flush output buffer")?;

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
