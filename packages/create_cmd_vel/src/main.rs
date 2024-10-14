use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use futures::StreamExt;
use r2r::{create_bridge_interface::msg::DirectDrive, geometry_msgs::msg::Twist, Node, QosProfile};
use tokio::signal::unix::{signal, SignalKind};

const MAX_VELOCITY_MMS: f64 = 500.0;
const AXEL_LENGTH_MM: f64 = 235.0;
const MAX_ANGULAR_VELOCITY_RADS: f64 =
    (2.0 * std::f64::consts::PI * AXEL_LENGTH_MM / 2.0) / MAX_VELOCITY_MMS;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = r2r::Context::create().context("Failed to create ROS context")?;
    let node = r2r::Node::create(ctx, "create_cmd_vel", "create_cmd_vel")
        .context("Failed to create ROS node")?;
    let log_name = node.logger().to_string();

    if let Err(error) = main_trampoline(node).await {
        r2r::log_error!(&log_name, "Fatal error: {error}");
    }

    Ok(())
}

async fn main_trampoline(mut node: Node) -> Result<()> {
    let mut velocity_request = node.subscribe::<Twist>("/cmd_vel", QosProfile::default())?;

    let drive =
        node.create_publisher::<DirectDrive>("/create_bridge/drive/direct", QosProfile::default())?;

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
            velocity_request = velocity_request.next() => {
                let velocity_request = velocity_request.unwrap();

                let linear = velocity_request.linear.x;
                let angular = velocity_request.angular.z;

                let linear = linear * MAX_VELOCITY_MMS;
                let angular = angular * MAX_ANGULAR_VELOCITY_RADS;

                let left_wheel_velocity = (linear - ((AXEL_LENGTH_MM / 2.0) * angular)) as i16;
                let right_wheel_velocity = (linear + ((AXEL_LENGTH_MM / 2.0) * angular)) as i16;

                drive.publish(&DirectDrive { left_wheel_velocity, right_wheel_velocity })?;
            }
        }
    }

    shutdown.store(true, Ordering::SeqCst);
    spin_handle.await.context("ROS spinner panicked")?;

    Ok(())
}
