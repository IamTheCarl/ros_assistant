pub mod roomba_interface;

use anyhow::{Context, Result};
use roomba_interface::{DriveCommand, LedState, Roomba, Sensor, TurnDirection};
use std::time::Duration;
use tokio::time::sleep;
use tokio_serial::SerialStream;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO the path and baud rate should be ROS params.
    let serial_config = tokio_serial::new(
        // "/dev/serial/by-id/usb-FTDI_FT231X_USB_UART_DA01NM8I-if00-port0",
        "/dev/ttyUSB0",
        115200,
    );

    let serial_interface =
        SerialStream::open(&serial_config).context("Failed to open serial port")?;

    let (read, write) = tokio::io::split(serial_interface);

    let mut roomba = Roomba::new(read, write).await?;

    roomba.drive(DriveCommand::Straight(-50)).await?;
    roomba.flush().await?;
    sleep(Duration::from_secs(5)).await;

    roomba
        .set_leds(LedState {
            check_robot: true,
            dock: true,
            spot: true,
            debris: true,
            power_color: 128,
            power_intensity: 255,
        })
        .await?;
    roomba
        .drive(DriveCommand::Turn(TurnDirection::Left(188)))
        .await?;
    roomba.flush().await?;
    sleep(Duration::from_secs(2)).await;

    roomba.drive(DriveCommand::Stop).await?;
    roomba.flush().await?;

    let text = "0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for i in 0..text.len() {
        roomba.set_seven_segment(&text[i..]).await?;
        roomba.flush().await?;
        sleep(Duration::from_millis(200)).await;
    }

    roomba.drive(DriveCommand::Stop).await?;
    roomba.flush().await?;

    roomba.start_stream(&[Sensor::BumpersAndWheelDrops]).await?;
    roomba.flush().await?;

    let mut sensor_stream = roomba
        .take_sensor_stream()
        .context("Sensor stream has already been taken.")?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            sensor_result = sensor_stream.recv() => {
                let sensor_data = sensor_result
                    .context("Error receiving sensor data.")?
                    .context("Error reading sensor data.");

                dbg!(sensor_data).ok();
            }
        }
    }

    roomba.set_seven_segment("Bye").await?;
    roomba
        .drive(DriveCommand::Turn(TurnDirection::Right(188)))
        .await?;
    roomba.flush().await?;
    sleep(Duration::from_secs(2)).await;

    roomba.seek_dock().await?;
    roomba.flush().await?;

    roomba.close().await?;

    Ok(())
}
