use anyhow::{bail, Context, Result};
use bytes::Bytes;
use futures::StreamExt;
use hashbrown::{HashMap, HashSet};
use r2r::{Node, PublisherUntyped, QosProfile};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::IsTerminal,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, DuplexStream, Stdin, Stdout},
    signal::unix::{signal, SignalKind},
    sync::{mpsc, oneshot},
    time::{Instant, MissedTickBehavior},
};

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();

    let mut output = io::stdout();
    let mut input = io::stdin();

    if std::io::stdout().is_terminal() {
        bail!("Refusing to output binary data to terminal");
    }

    log::info!("Starting tunnel.");

    start_communications(&mut output, &mut input)
        .await
        .context("Failed to start communications")?;

    tunnel_ros(&mut output, input)
        .await
        .context("Failed to tunnel ROS")?;

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

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_view = shutdown.clone();
    let spin_handle =
        tokio::task::spawn_blocking(move || while !shutdown_view.load(Ordering::SeqCst) {});

    let mut update_interval = tokio::time::interval_at(Instant::now(), Duration::from_secs(10));
    update_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut spin_interval = tokio::time::interval_at(Instant::now(), Duration::from_millis(100));
    update_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut sig_terminate =
        signal(SignalKind::terminate()).context("Failed to hook into terminate signal.")?;
    let mut sig_interrupt =
        signal(SignalKind::interrupt()).context("Failed to hook into interrupt signal.")?;

    let mut output_buffer = Vec::new();
    let mut input_buffer = Vec::new();

    let (incoming_message_tx, mut incoming_message_rx) = mpsc::channel(10);
    let (outgoing_message_tx, mut outgoing_message_rx) = mpsc::channel::<Bytes>(10);

    let message_reception = tokio::spawn(async move {
        loop {
            let message = input.recv_message::<Message>(&mut input_buffer).await;
            if let Err(error) = incoming_message_tx.send(message).await {
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
            message = incoming_message_rx.recv() => {
                let message = message.context("Message queue closed")?.context("Failed to receive message")?;
                let control_flow = process_command(
                        &mut node,
                        &known_topics,
                        &outgoing_message_tx,
                        &mut subscribers,
                        &mut publishers,
                        message)
                    .context("Failed to process message from remote")?;

                match control_flow {
                    ControlFlow::Continue => continue,
                    ControlFlow::Exit => break,
                }
            }
            message = outgoing_message_rx.recv() => {
                let message = message.unwrap();

                output.write_all(&message).await.context("Failed to write outgoing message to stdout")?;
                output.flush().await.context("Failed to flush stdout")?;
            }
            _ = update_interval.tick() => {
                scan_for_topics(&mut node, output, &mut output_buffer, &mut known_topics).await?;
                scan_for_subscriptions(output, &mut output_buffer, &publishers, &mut currently_subscribed).await?;
            }
            _ = spin_interval.tick() => {
                node.spin_once(std::time::Duration::from_millis(0));
            }
        }
    }

    // We're done with that.
    message_reception.abort();

    shutdown.store(true, Ordering::SeqCst);
    spin_handle.await.context("ROS spinner panicked")?;

    Ok(())
}

enum ControlFlow {
    Continue,
    Exit,
}

fn process_command(
    node: &mut Node,
    known_topics: &HashMap<String, String>,
    outgoing_message_tx: &mpsc::Sender<Bytes>,
    subscribers: &mut HashMap<String, oneshot::Sender<()>>,
    publishers: &mut HashMap<String, PublisherUntyped>,
    message: Message,
) -> Result<ControlFlow> {
    dbg!(&message);
    match message {
        Message::FatalError { message } => bail!("Fatal error from remote: {message}"),
        Message::Hangup => return Ok(ControlFlow::Exit),
        Message::NewTopic { name, message_type } => {
            if !publishers.contains_key(&name) {
                let result =
                    node.create_publisher_untyped(&name, &message_type, QosProfile::default());

                match result {
                    Ok(publisher) => {
                        publishers.insert(name, publisher);
                    }
                    Err(error) => {
                        log::warn!("Failed to create publisher: {error:?}");
                    }
                }
            } else {
                log::warn!("Remote has informed us of topic `{name}` being created twice.");
            }
        }
        Message::SetTopicSubscribed { name, subscribed } => {
            if subscribed {
                // Remote wants to subscribe, which means we need to subscribe.
                if let Some(topic_type) = known_topics.get(&name) {
                    match node.subscribe_raw(&name, topic_type, QosProfile::default()) {
                        Ok(mut subscription) => {
                            let (shutdown_tx, shutdown_rx) = oneshot::channel();
                            let mut outgoing_message_tx = outgoing_message_tx.clone();

                            tokio::spawn(async move {
                                let mut buffer = Vec::new();
                                tokio::select! {
                                    _ = shutdown_rx => {
                                        // That's our signal to unsubscribe.
                                        // This will break the loop below.
                                    }
                                    _ = async move {
                                        // This loop only exits if the ROS subscription stops providing us messages.
                                        while let Some(message) = subscription.next().await {
                                            if let Err(error) = outgoing_message_tx.send_message(&mut buffer, &message).await {
                                                log::error!("Failed to forward ros topic message to remote: {error:?}");
                                            }
                                        }
                                    } => {
                                        // ROS closed the stream.
                                    }
                                }
                            });

                            subscribers.insert(name, shutdown_tx);
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
                if let Some(shutdown_tx) = subscribers.remove(&name) {
                    // If this fails, it just means the task was already shutdown.
                    // That can happen if ROS stops providing the stream.
                    shutdown_tx.send(()).ok();
                } else {
                    log::error!("Remote wanted to unsubscribe from a topic `{name}`, but we were never subscribed to that.");
                }
            }
        }
        Message::DeletedTopic { name } => {
            if publishers.remove(&name).is_none() {
                log::warn!("Remote has informed us of the removal of topic `{name}`, but we were never aware of such a topic.");
            }
        }
        Message::PublishToTopic { name, payload } => {
            if let Some(publisher) = publishers.get(&name) {
                if let Err(error) = publisher.publish_raw(&payload) {
                    log::error!("Failed to publish to topic `{name}`: {error:?}");
                }
            } else {
                log::warn!("Remote has published to unknown topic `{name}`.");
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
            log::debug!("Topics: {topics:?}");

            // Remove dead topics.
            for (name, _message_type) in
                known_topics.extract_if(|name, _message_type| !topics.contains_key(name))
            {
                dbg!(&name, &_message_type);
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
                        output
                            .send_message(buffer, &Message::NewTopic { name, message_type })
                            .await
                            .context("Failed to send new message notice.")?;
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

#[derive(Debug, Serialize, Deserialize)]
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

    // Publish data over a topic.
    PublishToTopic { name: String, payload: Bytes },
}

type LengthIndicator = u32;

trait SendMessage {
    async fn send_message(
        &mut self,
        buffer: &mut Vec<u8>,
        message: &(impl Serialize + std::fmt::Debug),
    ) -> Result<()>;
}

fn encode_message(
    buffer: &mut Vec<u8>,
    message: &(impl Serialize + std::fmt::Debug),
) -> Result<()> {
    dbg!(message);
    buffer.clear();

    // Create a spacer for the length indicator.
    let length: LengthIndicator = 0;
    buffer.extend(length.to_le_bytes());

    // Write the payload to the buffer.
    bincode::serialize_into(&mut *buffer, message).context("Failed to serialize message")?;

    // Now we know the actual length of the message.
    let length: u32 = (buffer.len() - std::mem::size_of::<LengthIndicator>()) as LengthIndicator;
    buffer[0..std::mem::size_of::<LengthIndicator>()].copy_from_slice(&length.to_le_bytes());

    Ok(())
}

// I couldn't apply SendMessage generically to all AsyncWrite types, because
// apparently `mpsc::Sender<Bytes>` may implement AsyncWrite some day. I can't
// exclude it specifically from my implementation, so I had to do this.
macro_rules! impl_send_message {
    ($ty:ident) => {
        impl SendMessage for $ty {
            async fn send_message(
                &mut self,
                buffer: &mut Vec<u8>,
                message: &(impl Serialize + std::fmt::Debug),
            ) -> Result<()> {
                encode_message(buffer, message)?;

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
    };
}

// Sends messages through stdout, directly.
impl_send_message!(Stdout);
impl_send_message!(DuplexStream);

// External tasks can queue messages to be sent through stdout by the main task.
impl SendMessage for mpsc::Sender<Bytes> {
    async fn send_message(
        &mut self,
        _buffer: &mut Vec<u8>,
        message: &(impl Serialize + std::fmt::Debug),
    ) -> Result<()> {
        let mut buffer = Vec::new();
        encode_message(&mut buffer, message)?;

        self.send(buffer.into())
            .await
            .context("Outgoing message queue closed")?;

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
