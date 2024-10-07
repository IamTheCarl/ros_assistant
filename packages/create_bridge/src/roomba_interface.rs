use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{mpsc, Notify},
    task,
    task::JoinHandle,
    time::sleep,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Error shutting down sensor task: {0}")]
    Shutdown(#[from] task::JoinError),

    #[error("Requested a drive command with the speed or arch angle out of valid range.")]
    DriveRange,

    #[error("Roomba sent data for an unsupported sensor type: {0}")]
    InvalidSensorTypeID(u8),

    #[error("Invalid battery state: {0}")]
    InvalidBatteryState(u8),

    #[error("Invalid OI mode: {0}")]
    InvalidOIMode(u8),

    #[error("Unexpected end of message.")]
    UnexpectedEnd,
}

pub struct Roomba<
    ReadStream: AsyncRead + std::marker::Unpin + Send + 'static,
    WriteStream: AsyncWrite + std::marker::Unpin,
> {
    write_stream: WriteStream,
    _read_stream: std::marker::PhantomData<ReadStream>,
    _sensor_task: Option<JoinHandle<()>>,
    sensor_rx: Option<mpsc::Receiver<Result<SensorData, Error>>>,
    shutdown_notice: Arc<Notify>,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Sensor {
    BumpersAndWheelDrops = 7,
    Wall = 8,
    CliffLeft = 9,
    CliffFrontLeft = 10,
    CliffFrontRight = 11,
    CliffRight = 12,
    VirtualWall = 13,
    WheelOvercurrents = 14,
    DirtDetect = 15,
    InfraredCharacterOmni = 16,
    InfraredCharacterLeft = 52,
    InfraredCharacterRight = 53,
    Buttons = 18,
    Distance = 19,
    Angle = 20,
    ChargingState = 21,
    Voltage = 22,
    Current = 23,
    BatteryTemperature = 24,
    BatteryCharge = 25,
    BatteryCapacity = 26,
    WallSignal = 27,
    CliffLeftSignal = 28,
    CliffFrontLeftSignal = 29,
    CliffFrontRightSignal = 30,
    CliffRightSignal = 31,
    ChargingSourcesAvailable = 34,
    OIMode = 35,
    SongNumber = 36,
    SongPlaying = 37,
    NumberOfStreamPackets = 38,
    RequestedVelocity = 39,
    RequestedRadius = 40,
    RequestedRightVelocity = 41,
    RequestedLeftVelocity = 42,
    LeftEncoderCounts = 43,
    RightEncoderCounts = 44,
    LightBumper = 45,
    LightBumpLeftSignal = 46,
    LightBumpFrontLeftSignal = 47,
    LightBumpCenterLeftSignal = 48,
    LightBumpCenterRightSignal = 49,
    LightBumpFrontRightSignal = 50,
    LightBumpRightSignal = 51,
    LeftMotorCurrent = 54,
    RightMotorCurrent = 55,
    MainBrushMotorCurrent = 56,
    SideBrushMotorCurrent = 57,
    IsMovingForward = 58,
}

#[derive(Debug)]
pub enum SensorData {
    BumpersAndWheelDrops {
        wheel_drop_left: bool,
        wheel_drop_right: bool,
        bumper_left: bool,
        bumper_right: bool,
    },
    Wall(bool),
    CliffLeft(bool),
    CliffFrontLeft(bool),
    CliffFrontRight(bool),
    CliffRight(bool),
    VirtualWall(bool),
    WheelOvercurrents {
        left_wheel: bool,
        right_wheel: bool,
        main_brush: bool,
        side_brush: bool,
    },
    DirtDetect(u8),
    InfraredCharacterOmni(u8),
    InfraredCharacterLeft(u8),
    InfraredCharacterRight(u8),
    Buttons {
        clock: bool,
        schedule: bool,
        day: bool,
        hour: bool,
        minute: bool,
        dock: bool,
        spot: bool,
        clean: bool,
    },
    /// In millimeters.
    Distance(i16),
    /// Counter clockwise is negative, clockwise is positive, in millimeters.
    Angle(i16),
    ChargingState(ChargingState),
    /// In millivolts.
    Voltage(u16),
    /// In milliamps
    Current(i16),
    /// In Celsius
    BatteryTemperature(i8),
    /// In mAh
    BatteryCharge(u16),
    /// In mAh
    BatteryCapacity(u16),
    /// Strength of signal with 0 at 0% and 1023 at 100%.
    WallSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    CliffLeftSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    CliffFrontLeftSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    CliffFrontRightSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    CliffRightSignal(u16),
    ChargingSourcesAvailable {
        home_base: bool,
        internal_charger: bool,
    },
    OIMode(OIMode),
    SongNumber(u8),
    SongPlaying(bool),
    NumberOfStreamPackets(u8),
    /// In mm/s
    RequestedVelocity(i16),
    /// In millimeters
    RequestedRadius(i16),
    /// In mm/s
    RequestedRightVelocity(i16),
    /// In mm/s
    RequestedLeftVelocity(i16),
    LeftEncoderCounts(u16),
    RightEncoderCounts(u16),
    LightBumper {
        right: bool,
        front_right: bool,
        center_right: bool,
        center_left: bool,
        front_left: bool,
        left: bool,
    },
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpLeftSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpFrontLeftSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpCenterLeftSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpCenterRightSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpFrontRightSignal(u16),
    /// Strength of signal with 0 at 0% and 4095 at 100%.
    LightBumpRightSignal(u16),
    /// In mA
    LeftMotorCurrent(i16),
    /// In mA
    RightMotorCurrent(i16),
    /// In mA
    MainBrushMotorCurrent(i16),
    /// In mA
    SideBrushMotorCurrent(i16),
    /// The manual calls this "stasis" for some reason.
    IsMovingForward(bool),
}

// TODO we need a list of Infrared codes.

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OIMode {
    Off = 0,
    Passive = 1,
    Safe = 2,
    Full = 3,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ChargingState {
    NotCharging = 0,
    ReconditioningCharging = 1,
    FullCharging = 2,
    TrickleCharging = 3,
    Waiting = 4,
    ChargingFaultCondition = 5,
}

pub struct LedState {
    pub check_robot: bool,
    pub dock: bool,
    pub spot: bool,
    pub debris: bool,

    /// 0 is full green, 255 is full red, 128 is yellow.
    pub power_color: u8,
    pub power_intensity: u8,
}

pub enum TurnDirection {
    Left(u16),
    Right(u16),
}

/// Instructions on how the robot should drive.
/// Speed must be between -500 to +500 mm/s
/// Turn direction/radius can be between -2000 to 2000 mm.
pub enum DriveCommand {
    Straight(i16),
    Turn(TurnDirection),
    Arc { radius: TurnDirection, speed: i16 },
    Stop,
}

impl<
        ReadStream: AsyncRead + std::marker::Unpin + Send + 'static,
        WriteStream: AsyncWrite + std::marker::Unpin,
    > Roomba<ReadStream, WriteStream>
{
    pub async fn new<'a>(
        read_stream: ReadStream,
        write_stream: WriteStream,
    ) -> Result<Roomba<ReadStream, WriteStream>, Error> {
        let shutdown_notice = Arc::new(Notify::new());
        let (sensor_tx, sensor_rx) = mpsc::channel(10);

        let sensor_task = {
            let shutdown_notice = shutdown_notice.clone();

            tokio::spawn(async move {
                // We're going to do a lot of small reads, which is a bad idea for a lot of IO streams, so let's buffer it.
                let mut read_stream = BufReader::new(read_stream);

                let mut prelude = [0u8];
                let mut payload = Vec::new();

                async fn read<ReadStream: AsyncRead + std::marker::Unpin + Send + 'static>(
                    shutdown_notice: &Notify,
                    read_stream: &mut ReadStream,
                    payload: &mut [u8],
                ) -> Option<Result<(), Error>> {
                    tokio::select! {
                        _ = shutdown_notice.notified() => {
                            None
                        },
                        result = read_stream.read_exact(payload) => {
                            if let Err(error) = result {
                                Some(Err(error.into()))
                            } else {
                                Some(Ok(()))
                            }
                        }
                    }
                }

                while let Some(result) =
                    read(&shutdown_notice, &mut read_stream, &mut prelude).await
                {
                    // let debug = String::from_utf8_lossy(&prelude);
                    // print!("{}", debug);
                    // println!("READ: {:?}", prelude);
                    // println!("TEXT: {}", debug);

                    if let Err(error) = result {
                        sensor_tx.send(Err(error)).await.ok();
                    } else if prelude[0] == 19 {
                        // We got the magic number! Time to read a packet.

                        let mut length = [0u8];

                        if let Some(result) =
                            read(&shutdown_notice, &mut read_stream, &mut length).await
                        {
                            if let Err(error) = result {
                                sensor_tx.send(Err(error)).await.ok();
                            } else {
                                let length = length[0];
                                payload.resize(length as usize + 1, 0u8);

                                if let Some(result) =
                                    read(&shutdown_notice, &mut read_stream, &mut payload).await
                                {
                                    if let Err(error) = result {
                                        sensor_tx.send(Err(error)).await.ok();
                                    } else {
                                        // Check the sum.
                                        let checksum = payload
                                            .iter()
                                            .chain([19u8, length].iter())
                                            .fold(0u8, |acc, b| acc.wrapping_add(*b));
                                        if checksum == 0 {
                                            // We finally have a payload.
                                            // We're going to iterate it.
                                            let mut payload =
                                                payload[..payload.len() - 1].iter().copied();

                                            // We go until we run out of sensor IDs.
                                            while let Some(sensor_id) = payload.next() {
                                                if let Ok(sensor_id) =
                                                    Sensor::try_from_primitive(sensor_id)
                                                {
                                                    let sensor_data =
                                                        parse_sensor_data(sensor_id, &mut payload);

                                                    match sensor_data {
                                                        Ok(sensor_data) => {
                                                            sensor_tx
                                                                .send(Ok(sensor_data))
                                                                .await
                                                                .ok();
                                                        }
                                                        Err(error) => {
                                                            sensor_tx.send(Err(error)).await.ok();
                                                        }
                                                    }
                                                } else {
                                                    sensor_tx
                                                        .send(Err(Error::InvalidSensorTypeID(
                                                            sensor_id,
                                                        )))
                                                        .await
                                                        .ok();
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // We are shutting down.
                                    break;
                                }
                            }
                        } else {
                            // We are shutting down.
                            break;
                        }
                    }
                }
            })
        };

        let mut roomba = Roomba {
            write_stream,
            _read_stream: std::marker::PhantomData::default(),
            _sensor_task: Some(sensor_task),
            sensor_rx: Some(sensor_rx),
            shutdown_notice,
        };

        roomba.reset().await?;
        roomba.start().await?;

        Ok(roomba)
    }

    async fn start(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[128]).await?;

        Ok(())
    }

    async fn reset(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[7]).await?;
        sleep(Duration::from_secs(5)).await;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[173]).await?;

        Ok(())
    }

    async fn take_control(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[131]).await?;

        Ok(())
    }

    pub async fn clean(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[135]).await?;

        Ok(())
    }

    pub async fn spot(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[134]).await?;

        Ok(())
    }

    pub async fn seek_dock(&mut self) -> Result<(), Error> {
        self.write_stream.write_all(&[143]).await?;

        Ok(())
    }

    pub async fn drive(&mut self, command: DriveCommand) -> Result<(), Error> {
        // This command only works in safe or full mode.
        self.take_control().await?;

        let (velocity, radius) = match command {
            DriveCommand::Straight(speed) => (speed, 0x7FFF),
            DriveCommand::Turn(turn) => {
                let (speed, turn) = match turn {
                    TurnDirection::Left(speed) => (speed as i16, 1),
                    TurnDirection::Right(speed) => (speed as i16, -1),
                };

                (speed, turn)
            }
            DriveCommand::Arc { radius, speed } => {
                let radius = match radius {
                    TurnDirection::Left(speed) => speed as i16,
                    TurnDirection::Right(speed) => -(speed as i16),
                };

                (speed, radius)
            }
            DriveCommand::Stop => (0, 0),
        };

        // Drive command.
        self.write_stream.write_all(&[137]).await?;
        self.write_stream.write_all(&velocity.to_be_bytes()).await?;
        self.write_stream.write_all(&radius.to_be_bytes()).await?;

        Ok(())
    }

    pub async fn drive_direct(
        &mut self,
        left_wheel_velocity: i16,
        right_wheel_velocity: i16,
    ) -> Result<(), Error> {
        self.take_control().await?;
        
        self.write_stream.write_all(&[145]).await?;
        self.write_stream
            .write_all(&right_wheel_velocity.to_be_bytes())
            .await?;
        self.write_stream
            .write_all(&left_wheel_velocity.to_be_bytes())
            .await?;

        Ok(())
    }

    pub async fn set_leds(&mut self, led_state: LedState) -> Result<(), Error> {
        let mut leds = 0x00u8;

        if led_state.check_robot {
            leds |= 0x08;
        }

        if led_state.dock {
            leds |= 0x04;
        }

        if led_state.spot {
            leds |= 0x02;
        }

        if led_state.debris {
            leds |= 0x01;
        }

        self.take_control().await?;
        self.write_stream
            .write_all(&[139, leds, led_state.power_color, led_state.power_intensity])
            .await?;

        Ok(())
    }

    /// Note that while this will happily accept a full utf8 string, it can only
    /// display capitalized alphanumeric text (don't worry it'll auto capitalize for you) plus spaces.
    /// The display also only has 4 digits on it, so only the first 4 characters of any
    /// string will be displayed.
    pub async fn set_seven_segment(&mut self, text: &str) -> Result<(), Error> {
        let mut display_bytes = [b' '; 4];

        for (b, c) in display_bytes.iter_mut().zip(text.chars()) {
            *b = c as u8;
        }

        self.take_control().await?;

        self.write_stream.write_all(&[164]).await?;
        self.write_stream.write_all(&display_bytes).await?;

        Ok(())
    }

    /// Take the sensor stream for this Roomba. Will stream messages sent up by the Roomba.
    pub fn take_sensor_stream(&mut self) -> Option<mpsc::Receiver<Result<SensorData, Error>>> {
        self.sensor_rx.take()
    }

    /// Query a single sensor.
    /// You can get the result through the sensor stream provided by `take_sensor_stream`.
    pub async fn query(&mut self, sensor: Sensor) -> Result<(), Error> {
        self.take_control().await?;

        self.write_stream.write_all(&[142, sensor.into()]).await?;
        Ok(())
    }

    /// Query a list of sensors.
    /// You can get the result through the sensor stream provided by `take_sensor_stream`.
    pub async fn query_list(&mut self, sensors: &[Sensor]) -> Result<(), Error> {
        self.take_control().await?;

        self.write_stream
            .write_all(&[142, sensors.len() as u8])
            .await?;

        for sensor in sensors {
            self.write_stream.write_all(&[(*sensor).into()]).await?;
        }

        Ok(())
    }

    /// Start a stream of sensor data.
    /// You can get the results through the sensor stream provided by `take_sensor_stream`.
    pub async fn start_stream(&mut self, sensors: &[Sensor]) -> Result<(), Error> {
        self.take_control().await?;

        self.write_stream
            .write_all(&[148, sensors.len() as u8])
            .await?;

        for sensor in sensors.iter().copied() {
            self.write_stream.write_all(&[sensor.into()]).await?;
        }

        Ok(())
    }

    /// Set true to pause the stream, and false to resume.
    pub async fn pause_stream(&mut self, paused: bool) -> Result<(), Error> {
        let paused = if paused { 0x00 } else { 0x01 };

        self.write_stream.write_all(&[150, paused]).await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        self.write_stream.flush().await?;

        Ok(())
    }

    pub async fn close(mut self) -> Result<(), Error> {
        self.stop().await?;
        self.flush().await?;

        self.shutdown_notice.notify_waiters();

        // if let Some(sensor_task) = self.sensor_task.take() {
        //     sensor_task.await?;
        // }

        std::mem::forget(self);

        Ok(())
    }
}

impl<
        ReadStream: AsyncRead + std::marker::Unpin + Send + 'static,
        WriteStream: AsyncWrite + std::marker::Unpin,
    > Drop for Roomba<ReadStream, WriteStream>
{
    fn drop(&mut self) {
        println!("Improper drop of Roomba controller. Call Roomba::close() when you are done with a Roomba. Not doing this can result in battery damage.");
    }
}

fn parse_sensor_data(
    sensor_id: Sensor,
    payload: &mut impl Iterator<Item = u8>,
) -> Result<SensorData, Error> {
    fn too_short(option: Option<u8>) -> Result<u8, Error> {
        match option {
            Some(option) => Ok(option),
            None => Err(Error::UnexpectedEnd),
        }
    }

    fn single_bool(payload: &mut impl Iterator<Item = u8>) -> Result<bool, Error> {
        let status = too_short(payload.next())?;

        Ok(status & 0x01 != 0)
    }

    fn take_i16(payload: &mut impl Iterator<Item = u8>) -> Result<i16, Error> {
        let bytes = [too_short(payload.next())?, too_short(payload.next())?];

        Ok(i16::from_be_bytes(bytes))
    }

    fn take_u16(payload: &mut impl Iterator<Item = u8>) -> Result<u16, Error> {
        let bytes = [too_short(payload.next())?, too_short(payload.next())?];

        Ok(u16::from_be_bytes(bytes))
    }

    match sensor_id {
        Sensor::BumpersAndWheelDrops => {
            let status = too_short(payload.next())?;

            Ok(SensorData::BumpersAndWheelDrops {
                wheel_drop_left: status & 0x08 == 0,
                wheel_drop_right: status & 0x04 == 0,
                bumper_left: status & 0x02 != 0,
                bumper_right: status & 0x01 != 0,
            })
        }
        Sensor::Wall => Ok(SensorData::Wall(single_bool(payload)?)),
        Sensor::CliffLeft => Ok(SensorData::CliffLeft(single_bool(payload)?)),
        Sensor::CliffFrontLeft => Ok(SensorData::CliffFrontLeft(single_bool(payload)?)),
        Sensor::CliffFrontRight => Ok(SensorData::CliffFrontRight(single_bool(payload)?)),
        Sensor::CliffRight => Ok(SensorData::CliffRight(single_bool(payload)?)),
        Sensor::VirtualWall => Ok(SensorData::VirtualWall(single_bool(payload)?)),
        Sensor::WheelOvercurrents => {
            let state = too_short(payload.next())?;

            Ok(SensorData::WheelOvercurrents {
                left_wheel: state & 0x10 != 0,
                right_wheel: state & 0x08 != 0,
                main_brush: state & 0x04 != 0,
                side_brush: state & 0x01 != 0,
            })
        }
        Sensor::DirtDetect => Ok(SensorData::DirtDetect(too_short(payload.next())?)),
        Sensor::InfraredCharacterOmni => Ok(SensorData::InfraredCharacterOmni(too_short(
            payload.next(),
        )?)),
        Sensor::InfraredCharacterLeft => Ok(SensorData::InfraredCharacterLeft(too_short(
            payload.next(),
        )?)),
        Sensor::InfraredCharacterRight => Ok(SensorData::InfraredCharacterLeft(too_short(
            payload.next(),
        )?)),
        Sensor::Buttons => {
            let state = too_short(payload.next())?;

            Ok(SensorData::Buttons {
                clock: state & 0x80 != 0,
                schedule: state & 0x40 != 0,
                day: state & 0x20 != 0,
                hour: state & 0x10 != 0,
                minute: state & 0x08 != 0,
                dock: state & 0x04 != 0,
                spot: state & 0x02 != 0,
                clean: state & 0x01 != 0,
            })
        }
        Sensor::Distance => Ok(SensorData::Distance(take_i16(payload)?)),
        Sensor::Angle => Ok(SensorData::Angle(take_i16(payload)?)),
        Sensor::ChargingState => {
            let state = too_short(payload.next())?;
            if let Ok(state) = ChargingState::try_from(state) {
                Ok(SensorData::ChargingState(state))
            } else {
                Err(Error::InvalidBatteryState(state))
            }
        }
        Sensor::Voltage => Ok(SensorData::Voltage(take_u16(payload)?)),
        Sensor::Current => Ok(SensorData::Current(take_i16(payload)?)),
        Sensor::BatteryTemperature => Ok(SensorData::BatteryTemperature(
            too_short(payload.next())? as i8,
        )),
        Sensor::BatteryCharge => Ok(SensorData::BatteryCharge(take_u16(payload)?)),
        Sensor::BatteryCapacity => Ok(SensorData::BatteryCapacity(take_u16(payload)?)),
        Sensor::WallSignal => Ok(SensorData::WallSignal(take_u16(payload)?)),
        Sensor::CliffLeftSignal => Ok(SensorData::CliffLeftSignal(take_u16(payload)?)),
        Sensor::CliffFrontLeftSignal => Ok(SensorData::CliffFrontLeftSignal(take_u16(payload)?)),
        Sensor::CliffFrontRightSignal => Ok(SensorData::CliffFrontRightSignal(take_u16(payload)?)),
        Sensor::CliffRightSignal => Ok(SensorData::CliffRightSignal(take_u16(payload)?)),
        Sensor::ChargingSourcesAvailable => {
            let state = too_short(payload.next())?;

            Ok(SensorData::ChargingSourcesAvailable {
                home_base: state & 0x02 != 0,
                internal_charger: state & 0x01 != 0,
            })
        }
        Sensor::OIMode => {
            let state = too_short(payload.next())?;
            if let Ok(state) = OIMode::try_from(state) {
                Ok(SensorData::OIMode(state))
            } else {
                Err(Error::InvalidOIMode(state))
            }
        }
        Sensor::SongNumber => Ok(SensorData::SongNumber(too_short(payload.next())?)),
        Sensor::SongPlaying => Ok(SensorData::SongPlaying(too_short(payload.next())? != 0)),
        Sensor::NumberOfStreamPackets => Ok(SensorData::NumberOfStreamPackets(too_short(
            payload.next(),
        )?)),
        Sensor::RequestedVelocity => Ok(SensorData::RequestedVelocity(take_i16(payload)?)),
        Sensor::RequestedRadius => Ok(SensorData::RequestedRadius(take_i16(payload)?)),
        Sensor::RequestedRightVelocity => {
            Ok(SensorData::RequestedRightVelocity(take_i16(payload)?))
        }
        Sensor::RequestedLeftVelocity => Ok(SensorData::RequestedLeftVelocity(take_i16(payload)?)),
        Sensor::LeftEncoderCounts => Ok(SensorData::LeftEncoderCounts(take_u16(payload)?)),
        Sensor::RightEncoderCounts => Ok(SensorData::RightEncoderCounts(take_u16(payload)?)),
        Sensor::LightBumper => {
            let state = too_short(payload.next())?;

            Ok(SensorData::LightBumper {
                right: state & 0x20 != 0,
                front_right: state & 0x10 != 0,
                center_right: state & 0x08 != 0,
                center_left: state & 0x04 != 0,
                front_left: state & 0x02 != 0,
                left: state & 0x01 != 0,
            })
        }
        Sensor::LightBumpLeftSignal => {
            Ok(SensorData::LightBumpCenterLeftSignal(take_u16(payload)?))
        }
        Sensor::LightBumpFrontLeftSignal => {
            Ok(SensorData::LightBumpFrontLeftSignal(take_u16(payload)?))
        }
        Sensor::LightBumpCenterLeftSignal => {
            Ok(SensorData::LightBumpCenterLeftSignal(take_u16(payload)?))
        }
        Sensor::LightBumpCenterRightSignal => {
            Ok(SensorData::LightBumpCenterRightSignal(take_u16(payload)?))
        }
        Sensor::LightBumpFrontRightSignal => {
            Ok(SensorData::LightBumpFrontRightSignal(take_u16(payload)?))
        }
        Sensor::LightBumpRightSignal => Ok(SensorData::LightBumpRightSignal(take_u16(payload)?)),
        Sensor::LeftMotorCurrent => Ok(SensorData::LeftMotorCurrent(take_i16(payload)?)),
        Sensor::RightMotorCurrent => Ok(SensorData::RightMotorCurrent(take_i16(payload)?)),
        Sensor::MainBrushMotorCurrent => Ok(SensorData::MainBrushMotorCurrent(take_i16(payload)?)),
        Sensor::SideBrushMotorCurrent => Ok(SensorData::SideBrushMotorCurrent(take_i16(payload)?)),
        Sensor::IsMovingForward => Ok(SensorData::IsMovingForward(too_short(payload.next())? != 0)),
    }
}
