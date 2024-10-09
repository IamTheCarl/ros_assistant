use r2r::create_bridge_interface::msg::SensorQuery;

use crate::roomba_interface::Sensor;

pub fn query_list_from_ros_message(message: &SensorQuery) -> Vec<Sensor> {
    let mut sensor_list = Vec::new();

    if message.bumpers_and_wheel_drops {
        sensor_list.push(Sensor::BumpersAndWheelDrops);
    }
    if message.wall {
        sensor_list.push(Sensor::Wall);
    }
    if message.cliff_left {
        sensor_list.push(Sensor::CliffLeft);
    }
    if message.cliff_front_left {
        sensor_list.push(Sensor::CliffFrontLeft);
    }
    if message.cliff_front_right {
        sensor_list.push(Sensor::CliffFrontRight);
    }
    if message.cliff_right {
        sensor_list.push(Sensor::CliffRight);
    }
    if message.virtual_wall {
        sensor_list.push(Sensor::VirtualWall);
    }
    if message.wheel_overcurrents {
        sensor_list.push(Sensor::WheelOvercurrents);
    }
    if message.dirt_detect {
        sensor_list.push(Sensor::DirtDetect);
    }
    if message.infrared_character_omni {
        sensor_list.push(Sensor::InfraredCharacterOmni);
    }
    if message.infrared_character_left {
        sensor_list.push(Sensor::InfraredCharacterLeft);
    }
    if message.infrared_character_right {
        sensor_list.push(Sensor::InfraredCharacterRight);
    }
    if message.buttons {
        sensor_list.push(Sensor::Buttons);
    }
    if message.distance {
        sensor_list.push(Sensor::Distance);
    }
    if message.angle {
        sensor_list.push(Sensor::Angle);
    }
    if message.charging_state {
        sensor_list.push(Sensor::ChargingState);
    }
    if message.voltage {
        sensor_list.push(Sensor::Voltage);
    }
    if message.current {
        sensor_list.push(Sensor::Current);
    }
    if message.battery_temperature {
        sensor_list.push(Sensor::BatteryTemperature);
    }
    if message.battery_charge {
        sensor_list.push(Sensor::BatteryCharge);
    }
    if message.battery_capacity {
        sensor_list.push(Sensor::BatteryCapacity);
    }
    if message.wall_signal {
        sensor_list.push(Sensor::WallSignal);
    }
    if message.cliff_left_signal {
        sensor_list.push(Sensor::CliffLeftSignal);
    }
    if message.cliff_front_left_signal {
        sensor_list.push(Sensor::CliffFrontLeftSignal);
    }
    if message.cliff_front_right_signal {
        sensor_list.push(Sensor::CliffFrontRightSignal);
    }
    if message.cliff_right_signal {
        sensor_list.push(Sensor::CliffRightSignal);
    }
    if message.charging_sources_available {
        sensor_list.push(Sensor::ChargingSourcesAvailable);
    }
    if message.oi_mode {
        sensor_list.push(Sensor::OIMode);
    }
    if message.song_number {
        sensor_list.push(Sensor::SongNumber);
    }
    if message.song_playing {
        sensor_list.push(Sensor::SongPlaying);
    }
    if message.number_of_stream_packets {
        sensor_list.push(Sensor::NumberOfStreamPackets);
    }
    if message.requested_velocity {
        sensor_list.push(Sensor::RequestedVelocity);
    }
    if message.requested_radius {
        sensor_list.push(Sensor::RequestedRadius);
    }
    if message.requested_right_velocity {
        sensor_list.push(Sensor::RequestedRightVelocity);
    }
    if message.requested_left_velocity {
        sensor_list.push(Sensor::RequestedLeftVelocity);
    }
    if message.left_encoder_counts {
        sensor_list.push(Sensor::LeftEncoderCounts);
    }
    if message.right_encoder_counts {
        sensor_list.push(Sensor::RightEncoderCounts);
    }
    if message.light_bumper {
        sensor_list.push(Sensor::LightBumper);
    }
    if message.light_bump_left_signal {
        sensor_list.push(Sensor::LightBumpLeftSignal);
    }
    if message.light_bump_front_left_signal {
        sensor_list.push(Sensor::LightBumpFrontLeftSignal);
    }
    if message.light_bump_center_left_signal {
        sensor_list.push(Sensor::LightBumpCenterLeftSignal);
    }
    if message.light_bump_center_right_signal {
        sensor_list.push(Sensor::LightBumpCenterRightSignal);
    }
    if message.light_bump_front_right_signal {
        sensor_list.push(Sensor::LightBumpFrontRightSignal);
    }
    if message.light_bump_right_signal {
        sensor_list.push(Sensor::LightBumpRightSignal);
    }
    if message.left_motor_current {
        sensor_list.push(Sensor::LeftMotorCurrent);
    }
    if message.right_motor_current {
        sensor_list.push(Sensor::RightMotorCurrent);
    }
    if message.main_brush_motor_current {
        sensor_list.push(Sensor::MainBrushMotorCurrent);
    }
    if message.side_brush_motor_current {
        sensor_list.push(Sensor::SideBrushMotorCurrent);
    }
    if message.is_moving_forward {
        sensor_list.push(Sensor::IsMovingForward);
    }

    sensor_list
}
