use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use futures::stream::StreamExt;
use r2r::{
    create_bridge_interface::msg::{DirectDrive, DriveArc, LEDState, SensorQuery},
    std_msgs::msg::{Bool, Empty, Int16},
    Node, QosProfile,
};
use roomba_interface::{DriveCommand, LedState, Roomba, TurnDirection};
use sensors::SensorSet;
use tokio::{
    io::AsyncRead,
    signal::unix::{signal, SignalKind},
};
use tokio_serial::SerialStream;

pub mod roomba_interface;
mod sensors;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = r2r::Context::create().context("Failed to create ROS context")?;
    let node = r2r::Node::create(ctx, "create_bridge", "create_bridge")
        .context("Failed to create ROS node")?;
    let log_name = node.logger().to_string();

    if let Err(error) = main_trampoline(node, &log_name).await {
        r2r::log_error!(&log_name, "Fatal error: {error}");
    }

    Ok(())
}

async fn main_trampoline(node: Node, log_name: &str) -> Result<()> {
    let serial_device: String = node
        .get_parameter("serial_device")
        .context("Failed to get serial device name.")?;

    let baud_rate: Option<i64> = node
        .get_parameter("baud_rate")
        .context("Failed to get device baud rate.")?;
    let baud_rate = baud_rate.unwrap_or(115200);

    r2r::log_info!(
        log_name,
        "Opening serial interface {serial_device} with baud rate {baud_rate}"
    );

    let serial_config = tokio_serial::new(serial_device, baud_rate as u32);
    let serial_interface =
        SerialStream::open(&serial_config).context("Failed to open serial port")?;

    let (read, write) = tokio::io::split(serial_interface);
    let mut roomba = Roomba::new(read, write).await?;

    r2r::log_info!(log_name, "Interface opened.");

    if let Err(error) = roomba_trampoline(&mut roomba, node).await {
        r2r::log_error!(log_name, "Fatal error: {error}");
    }

    roomba.close().await?;

    Ok(())
}

async fn roomba_trampoline<R, W>(roomba: &mut Roomba<R, W>, mut node: Node) -> Result<()>
where
    R: AsyncRead + std::marker::Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + std::marker::Unpin,
{
    let mut clean_service = node.subscribe::<Empty>("clean", QosProfile::default())?;
    let mut spot_clean_service = node.subscribe::<Empty>("spot_clean", QosProfile::default())?;
    let mut dock_service = node.subscribe::<Empty>("dock", QosProfile::default())?;
    let mut led_state = node.subscribe::<LEDState>("led_state", QosProfile::default())?;
    let mut display_text =
        node.subscribe::<r2r::std_msgs::msg::String>("display_text", QosProfile::default())?;

    let mut drive_straight = node.subscribe::<Int16>("drive/straight", QosProfile::default())?;
    let mut drive_left = node.subscribe::<Int16>("drive/left", QosProfile::default())?;
    let mut drive_right = node.subscribe::<Int16>("drive/right", QosProfile::default())?;
    let mut drive_arc_left = node.subscribe::<DriveArc>("drive/arc_left", QosProfile::default())?;
    let mut drive_arc_right =
        node.subscribe::<DriveArc>("drive/arc_right", QosProfile::default())?;
    let mut drive_stop = node.subscribe::<Empty>("drive/stop", QosProfile::default())?;
    let mut direct_drive = node.subscribe::<DirectDrive>("drive/direct", QosProfile::default())?;

    let mut sensor_query = node.subscribe::<SensorQuery>("sensor/query", QosProfile::default())?;
    let mut sensor_start_stream =
        node.subscribe::<SensorQuery>("sensor/start_stream", QosProfile::default())?;
    let mut sensor_pause = node.subscribe::<Bool>("sensor/pause", QosProfile::default())?;

    let sensor_set = SensorSet::new(&mut node)?;
    let mut sensor_stream = roomba
        .take_sensor_stream()
        .expect("Sensor stream was already taken");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_view = shutdown.clone();

    let spin_handle = tokio::task::spawn_blocking(move || {
        while !shutdown_view.load(Ordering::SeqCst) {
            node.spin_once(std::time::Duration::from_millis(100));
        }
    });

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

            _ = clean_service.next() => {
                roomba.clean().await?;
            }
            _ = spot_clean_service.next() => {
                roomba.spot().await?;
            }
            _ = dock_service.next() => {
                roomba.seek_dock().await?;
            }
            new_led_state = led_state.next() => {
                let new_led_state = new_led_state.unwrap();
                let new_led_state = LedState {
                    check_robot: new_led_state.check_robot,
                    dock: new_led_state.dock,
                    spot: new_led_state.spot,
                    debris: new_led_state.debris,
                    power_color: new_led_state.power_color,
                    power_intensity: new_led_state.power_intensity,
                };
                roomba.set_leds(new_led_state).await?;
            }
            display_text = display_text.next() => {
                let display_text = display_text.unwrap();

                roomba.set_seven_segment(&display_text.data).await?;
            }

            drive_straight = drive_straight.next() => {
                let drive_straight = drive_straight.unwrap();

                roomba.drive(DriveCommand::Straight(drive_straight.data)).await?;
            }
            drive_left = drive_left.next() => {
                let drive_left = drive_left.unwrap();

                // Values less than 0 are invalid, so we'll just default them to zero.
                let value = drive_left.data.max(0) as u16;

                roomba.drive(DriveCommand::Turn(TurnDirection::Left(value))).await?;
            }
            drive_right = drive_right.next() => {
                let drive_right = drive_right.unwrap();

                // Values less than 0 are invalid, so we'll just default them to zero.
                let value = drive_right.data.max(0) as u16;

                roomba.drive(DriveCommand::Turn(TurnDirection::Right(value))).await?;
            }
            drive_arc_left = drive_arc_left.next() => {
                let drive_arc = drive_arc_left.unwrap();

                let speed = drive_arc.speed;

                // Values less than 0 are invalid, so we'll just default them to zero.
                let radius = drive_arc.radius.max(0);
                let radius = TurnDirection::Left(radius as u16);

                roomba.drive(DriveCommand::Arc { radius, speed }).await?;
            }
            drive_arc_right = drive_arc_right.next() => {
                let drive_arc = drive_arc_right.unwrap();

                let speed = drive_arc.speed;

                // Values less than 0 are invalid, so we'll just default them to zero.
                let radius = drive_arc.radius.max(0);
                let radius = TurnDirection::Right(radius as u16);

                roomba.drive(DriveCommand::Arc { radius, speed }).await?;
            }
            _ = drive_stop.next() => {
                roomba.drive(DriveCommand::Stop).await?;
            }
            direct_drive = direct_drive.next() => {
                let direct_drive = direct_drive.unwrap();

                roomba.drive_direct(direct_drive.left_wheel_velocity, direct_drive.right_wheel_velocity).await?;
            }
            sensor_query = sensor_query.next() => {
                let sensor_query = sensor_query.unwrap();

                let sensor_list = sensors::query_list_from_ros_message(&sensor_query);
                roomba.query_list(&sensor_list).await?;
            }
            sensor_query = sensor_start_stream.next() => {
                let sensor_query = sensor_query.unwrap();

                let sensor_list = sensors::query_list_from_ros_message(&sensor_query);
                roomba.start_stream(&sensor_list).await?;
            }
            paused = sensor_pause.next() => {
                let paused = paused.unwrap();
                let paused = paused.data;

                roomba.pause_stream(paused).await?;
            }
            sensor_data = sensor_stream.recv() => {
                let sensor_data = sensor_data.unwrap()?;

                sensor_set.publish(sensor_data)?;
            }
        }

        roomba.flush().await?;
    }

    shutdown.store(true, Ordering::SeqCst);
    spin_handle.await.context("ROS spinner panicked")?;

    Ok(())
}
