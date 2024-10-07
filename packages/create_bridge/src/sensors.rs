use r2r::{
    create_bridge_interface::msg::{
        BumpersAndWheelDrops, Buttons, ChargingSourcesAvailable, ChargingState, LightBumper,
        OIMode, SensorQuery, WheelOvercurrents,
    },
    std_msgs::msg::{Bool, Int16, Int8, UInt16, UInt8},
    Node, Publisher, QosProfile, Result,
};

use crate::roomba_interface::{Sensor, SensorData};

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

pub struct SensorSet {
    bumpers_and_wheel_drops: Publisher<BumpersAndWheelDrops>,
    wall: Publisher<Bool>,
    cliff_left: Publisher<Bool>,
    cliff_front_left: Publisher<Bool>,
    cliff_front_right: Publisher<Bool>,
    cliff_right: Publisher<Bool>,
    virtual_wall: Publisher<Bool>,
    wheel_overcurrents: Publisher<WheelOvercurrents>,
    dirt_detect: Publisher<UInt8>,
    infrared_character_omni: Publisher<UInt8>,
    infrared_character_left: Publisher<UInt8>,
    infrared_character_right: Publisher<UInt8>,
    buttons: Publisher<Buttons>,
    distance: Publisher<Int16>,
    angle: Publisher<Int16>,
    charging_state: Publisher<ChargingState>,
    voltage: Publisher<UInt16>,
    current: Publisher<Int16>,
    battery_temperature: Publisher<Int8>,
    battery_charge: Publisher<UInt16>,
    battery_capacity: Publisher<UInt16>,
    wall_signal: Publisher<UInt16>,
    cliff_left_signal: Publisher<UInt16>,
    cliff_right_signal: Publisher<UInt16>,
    cliff_front_right_signal: Publisher<UInt16>,
    cliff_front_left_signal: Publisher<UInt16>,
    charging_sources_available: Publisher<ChargingSourcesAvailable>,
    oi_mode: Publisher<OIMode>,
    song_number: Publisher<UInt8>,
    song_playing: Publisher<Bool>,
    number_of_stream_packets: Publisher<UInt8>,
    requested_velocity: Publisher<Int16>,
    requested_radius: Publisher<Int16>,
    requested_right_velocity: Publisher<Int16>,
    requested_left_velocity: Publisher<Int16>,
    left_encoder_counts: Publisher<UInt16>,
    right_encoder_counts: Publisher<UInt16>,
    light_bumper: Publisher<LightBumper>,
    light_bump_left_signal: Publisher<UInt16>,
    light_bump_front_left_signal: Publisher<UInt16>,
    light_bump_center_left_signal: Publisher<UInt16>,
    light_bump_center_right_signal: Publisher<UInt16>,
    light_bump_front_right_signal: Publisher<UInt16>,
    light_bump_right_signal: Publisher<UInt16>,
    left_motor_current: Publisher<Int16>,
    right_motor_current: Publisher<Int16>,
    main_brush_motor_current: Publisher<Int16>,
    side_brush_motor_current: Publisher<Int16>,
    is_moving_forward: Publisher<Bool>,
}

impl SensorSet {
    pub fn new(node: &mut Node) -> Result<Self> {
        Ok(Self {
            bumpers_and_wheel_drops: node.create_publisher::<BumpersAndWheelDrops>(
                "sensors/bumpers_and_wheel_drops",
                QosProfile::default(),
            )?,
            wall: node.create_publisher::<Bool>("sensors/wall", QosProfile::default())?,
            cliff_left: node
                .create_publisher::<Bool>("sensors/cliff_left", QosProfile::default())?,
            cliff_front_left: node
                .create_publisher::<Bool>("sensors/cliff_front_left", QosProfile::default())?,
            cliff_front_right: node
                .create_publisher::<Bool>("sensors/cliff_front_right", QosProfile::default())?,
            cliff_right: node
                .create_publisher::<Bool>("sensors/cliff_right", QosProfile::default())?,
            virtual_wall: node
                .create_publisher::<Bool>("sensors/virtual_wall", QosProfile::default())?,
            wheel_overcurrents: node.create_publisher::<WheelOvercurrents>(
                "sensors/wheel_overcurrents",
                QosProfile::default(),
            )?,
            dirt_detect: node
                .create_publisher::<UInt8>("sensors/dirt_detect", QosProfile::default())?,
            infrared_character_omni: node.create_publisher::<UInt8>(
                "sensors/infrared_character_omni",
                QosProfile::default(),
            )?,
            infrared_character_left: node.create_publisher::<UInt8>(
                "sensors/infrared_character_left",
                QosProfile::default(),
            )?,
            infrared_character_right: node.create_publisher::<UInt8>(
                "sensors/infrared_character_right",
                QosProfile::default(),
            )?,
            buttons: node.create_publisher::<Buttons>("sensors/buttons", QosProfile::default())?,
            distance: node.create_publisher::<Int16>("sensors/distance", QosProfile::default())?,
            angle: node.create_publisher::<Int16>("sensors/angle", QosProfile::default())?,
            charging_state: node.create_publisher::<ChargingState>(
                "sensors/charging_state",
                QosProfile::default(),
            )?,
            voltage: node.create_publisher::<UInt16>("sensors/voltage", QosProfile::default())?,
            current: node.create_publisher::<Int16>("sensors/current", QosProfile::default())?,
            battery_temperature: node
                .create_publisher::<Int8>("sensors/battery_temperature", QosProfile::default())?,
            battery_charge: node
                .create_publisher::<UInt16>("sensors/battery_charge", QosProfile::default())?,
            battery_capacity: node
                .create_publisher::<UInt16>("sensors/battery_capacity", QosProfile::default())?,
            wall_signal: node
                .create_publisher::<UInt16>("sensors/wall_signal", QosProfile::default())?,
            cliff_left_signal: node
                .create_publisher::<UInt16>("sensors/cliff_left_signal", QosProfile::default())?,
            cliff_right_signal: node
                .create_publisher::<UInt16>("sensors/cliff_right_signal", QosProfile::default())?,
            cliff_front_right_signal: node.create_publisher::<UInt16>(
                "sensors/cliff_front_right_signal",
                QosProfile::default(),
            )?,
            cliff_front_left_signal: node.create_publisher::<UInt16>(
                "sensors/cliff_front_left_signal",
                QosProfile::default(),
            )?,
            charging_sources_available: node.create_publisher::<ChargingSourcesAvailable>(
                "sensors/charging_sources_available",
                QosProfile::default(),
            )?,
            oi_mode: node.create_publisher::<OIMode>("sensors/oi_mode", QosProfile::default())?,
            song_number: node
                .create_publisher::<UInt8>("sensors/song_number", QosProfile::default())?,
            song_playing: node
                .create_publisher::<Bool>("sensors/song_playing", QosProfile::default())?,
            number_of_stream_packets: node.create_publisher::<UInt8>(
                "sensors/number_of_stream_packets",
                QosProfile::default(),
            )?,
            requested_velocity: node
                .create_publisher::<Int16>("sensors/requested_velocity", QosProfile::default())?,
            requested_radius: node
                .create_publisher::<Int16>("sensors/requested_radius", QosProfile::default())?,
            requested_right_velocity: node.create_publisher::<Int16>(
                "sensors/requested_right_velocity",
                QosProfile::default(),
            )?,
            requested_left_velocity: node.create_publisher::<Int16>(
                "sensors/requested_left_velocity",
                QosProfile::default(),
            )?,
            left_encoder_counts: node
                .create_publisher::<UInt16>("sensors/left_encoder_counts", QosProfile::default())?,
            right_encoder_counts: node.create_publisher::<UInt16>(
                "sensors/right_encoder_counts",
                QosProfile::default(),
            )?,
            light_bumper: node
                .create_publisher::<LightBumper>("sensors/light_bumper", QosProfile::default())?,
            light_bump_left_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_left_signal",
                QosProfile::default(),
            )?,
            light_bump_front_left_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_front_left_signal",
                QosProfile::default(),
            )?,
            light_bump_center_left_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_center_left_signal",
                QosProfile::default(),
            )?,
            light_bump_center_right_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_center_right_signal",
                QosProfile::default(),
            )?,
            light_bump_front_right_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_front_right_signal",
                QosProfile::default(),
            )?,
            light_bump_right_signal: node.create_publisher::<UInt16>(
                "sensors/light_bump_right_signal",
                QosProfile::default(),
            )?,
            left_motor_current: node
                .create_publisher::<Int16>("sensors/left_motor_current", QosProfile::default())?,
            right_motor_current: node
                .create_publisher::<Int16>("sensors/right_motor_current", QosProfile::default())?,
            main_brush_motor_current: node.create_publisher::<Int16>(
                "sensors/main_brush_motor_current",
                QosProfile::default(),
            )?,
            side_brush_motor_current: node.create_publisher::<Int16>(
                "sensors/side_brush_motor_current",
                QosProfile::default(),
            )?,
            is_moving_forward: node
                .create_publisher::<Bool>("sensors/is_moving_forward", QosProfile::default())?,
        })
    }

    pub fn publish(&self, data: SensorData) -> Result<()> {
        match data {
            SensorData::BumpersAndWheelDrops {
                wheel_drop_left,
                wheel_drop_right,
                bumper_left,
                bumper_right,
            } => self.bumpers_and_wheel_drops.publish(&BumpersAndWheelDrops {
                wheel_drop_left,
                wheel_drop_right,
                bumper_left,
                bumper_right,
            }),
            SensorData::Wall(data) => self.wall.publish(&Bool { data }),
            SensorData::CliffLeft(data) => self.cliff_left.publish(&Bool { data }),
            SensorData::CliffFrontLeft(data) => self.cliff_front_left.publish(&Bool { data }),
            SensorData::CliffFrontRight(data) => self.cliff_front_right.publish(&Bool { data }),
            SensorData::CliffRight(data) => self.cliff_right.publish(&Bool { data }),
            SensorData::VirtualWall(data) => self.virtual_wall.publish(&Bool { data }),
            SensorData::WheelOvercurrents {
                left_wheel,
                right_wheel,
                main_brush,
                side_brush,
            } => self.wheel_overcurrents.publish(&WheelOvercurrents {
                left_wheel,
                right_wheel,
                main_brush,
                side_brush,
            }),
            SensorData::DirtDetect(data) => self.dirt_detect.publish(&UInt8 { data }),
            SensorData::InfraredCharacterOmni(data) => {
                self.infrared_character_omni.publish(&UInt8 { data })
            }
            SensorData::InfraredCharacterLeft(data) => {
                self.infrared_character_left.publish(&UInt8 { data })
            }
            SensorData::InfraredCharacterRight(data) => {
                self.infrared_character_right.publish(&UInt8 { data })
            }
            SensorData::Buttons {
                clock,
                schedule,
                day,
                hour,
                minute,
                dock,
                spot,
                clean,
            } => self.buttons.publish(&Buttons {
                clock,
                schedule,
                day,
                hour,
                minute,
                dock,
                spot,
                clean,
            }),
            SensorData::Distance(data) => self.distance.publish(&Int16 { data }),
            SensorData::Angle(data) => self.angle.publish(&Int16 { data }),
            SensorData::ChargingState(state) => self.charging_state.publish(&ChargingState {
                state: state.into(),
            }),
            SensorData::Voltage(data) => self.voltage.publish(&UInt16 { data }),
            SensorData::Current(data) => self.current.publish(&Int16 { data }),
            SensorData::BatteryTemperature(data) => {
                self.battery_temperature.publish(&Int8 { data })
            }
            SensorData::BatteryCharge(data) => self.battery_charge.publish(&UInt16 { data }),
            SensorData::BatteryCapacity(data) => self.battery_capacity.publish(&UInt16 { data }),
            SensorData::WallSignal(data) => self.wall_signal.publish(&UInt16 { data }),
            SensorData::CliffLeftSignal(data) => self.cliff_left_signal.publish(&UInt16 { data }),
            SensorData::CliffFrontLeftSignal(data) => {
                self.cliff_front_left_signal.publish(&UInt16 { data })
            }
            SensorData::CliffFrontRightSignal(data) => {
                self.cliff_front_right_signal.publish(&UInt16 { data })
            }
            SensorData::CliffRightSignal(data) => self.cliff_right_signal.publish(&UInt16 { data }),
            SensorData::ChargingSourcesAvailable {
                home_base,
                internal_charger,
            } => self
                .charging_sources_available
                .publish(&ChargingSourcesAvailable {
                    home_base,
                    internal_charger,
                }),
            SensorData::OIMode(mode) => self.oi_mode.publish(&OIMode { mode: mode.into() }),
            SensorData::SongNumber(data) => self.song_number.publish(&UInt8 { data }),
            SensorData::SongPlaying(data) => self.song_playing.publish(&Bool { data }),
            SensorData::NumberOfStreamPackets(data) => {
                self.number_of_stream_packets.publish(&UInt8 { data })
            }
            SensorData::RequestedVelocity(data) => self.requested_velocity.publish(&Int16 { data }),
            SensorData::RequestedRadius(data) => self.requested_radius.publish(&Int16 { data }),
            SensorData::RequestedRightVelocity(data) => {
                self.requested_right_velocity.publish(&Int16 { data })
            }
            SensorData::RequestedLeftVelocity(data) => {
                self.requested_left_velocity.publish(&Int16 { data })
            }
            SensorData::LeftEncoderCounts(data) => {
                self.left_encoder_counts.publish(&UInt16 { data })
            }
            SensorData::RightEncoderCounts(data) => {
                self.right_encoder_counts.publish(&UInt16 { data })
            }
            SensorData::LightBumper {
                right,
                front_right,
                center_right,
                center_left,
                front_left,
                left,
            } => self.light_bumper.publish(&LightBumper {
                right,
                front_right,
                center_right,
                center_left,
                front_left,
                left,
            }),
            SensorData::LightBumpLeftSignal(data) => {
                self.light_bump_left_signal.publish(&UInt16 { data })
            }
            SensorData::LightBumpFrontLeftSignal(data) => {
                self.light_bump_front_left_signal.publish(&UInt16 { data })
            }
            SensorData::LightBumpCenterLeftSignal(data) => {
                self.light_bump_center_left_signal.publish(&UInt16 { data })
            }
            SensorData::LightBumpCenterRightSignal(data) => self
                .light_bump_center_right_signal
                .publish(&UInt16 { data }),
            SensorData::LightBumpFrontRightSignal(data) => {
                self.light_bump_front_right_signal.publish(&UInt16 { data })
            }
            SensorData::LightBumpRightSignal(data) => {
                self.light_bump_right_signal.publish(&UInt16 { data })
            }
            SensorData::LeftMotorCurrent(data) => self.left_motor_current.publish(&Int16 { data }),
            SensorData::RightMotorCurrent(data) => {
                self.right_motor_current.publish(&Int16 { data })
            }
            SensorData::MainBrushMotorCurrent(data) => {
                self.main_brush_motor_current.publish(&Int16 { data })
            }
            SensorData::SideBrushMotorCurrent(data) => {
                self.side_brush_motor_current.publish(&Int16 { data })
            }
            SensorData::IsMovingForward(data) => self.is_moving_forward.publish(&Bool { data }),
        }
    }
}
