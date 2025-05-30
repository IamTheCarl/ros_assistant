#![no_std]
#![no_main]

use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, StackResources};
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    peripherals,
    time::mhz,
    uid::{uid, uid_hex},
    usb::{self, Driver},
    Config,
};
use embassy_time::Timer;
use embassy_usb::{
    class::cdc_ncm::{
        embassy_net::{Device as NetDevice, Runner, State as NetState},
        CdcNcmClass, State as UsbState,
    },
    Builder, UsbDevice,
};
use heapless::Vec;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type UsbDriver = Driver<'static, embassy_stm32::peripherals::USB>;

const MTU: usize = 1514;

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    let p = embassy_stm32::init(config);
    defmt::info!("Hello World!");

    let driver = UsbDriver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("The Carl");
    config.product = Some("Robot Controller Board");
    config.serial_number = Some(uid_hex());
    // config.self_powered = true; // This most likely can't be used because our chip/board doesn't
    // support vbus detection.
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut [], // no msos descriptors
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Our MAC addr.
    let mut our_mac_addr = [0x00; 6];
    our_mac_addr.copy_from_slice(&uid()[6..12]);
    let first_byte = &mut our_mac_addr[0];
    *first_byte |= 0x02; // Make sure it's "locally administered".
    *first_byte &= !0x01; // Make sure it's not a multicast address.

    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let mut host_mac_addr = [0x00; 6];
    host_mac_addr.copy_from_slice(&our_mac_addr);
    // Increment the last byte to make sure this address is different from ours.

    // Create classes on the builder.
    static STATE: StaticCell<UsbState> = StaticCell::new();
    let class = CdcNcmClass::new(&mut builder, STATE.init(UsbState::new()), host_mac_addr, 64);

    // Build the builder.
    let usb = builder.build();

    defmt::unwrap!(spawner.spawn(usb_task(usb)));

    static NET_STATE: StaticCell<NetState<MTU, 4, 4>> = StaticCell::new();
    let net_state = NET_STATE.init_with(|| NetState::new());
    let (runner, device) = class.into_embassy_net_device::<MTU, 4, 4>(net_state, our_mac_addr);
    defmt::unwrap!(spawner.spawn(usb_ncm_task(runner)));

    // let config = embassy_net::Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(11, 11, 11, 2), 24),
        dns_servers: Vec::new(),
        gateway: None,
    });

    // We don't have a random number generator on this device, so we use the chip's UID as a seed.
    // TODO We should experiment with using the system time to improve the quality of the random
    // seed.
    let seed = {
        let mut seed = [0u8; 8];
        let uid_bytes = uid();
        seed.copy_from_slice(&uid_bytes[0..8]);
        u64::from_le_bytes(seed)
    };

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    defmt::unwrap!(spawner.spawn(net_task(runner)));

    let mut led = Output::new(p.PE13, Level::High, Speed::Low);

    loop {
        defmt::info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        defmt::info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, UsbDriver, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, NetDevice<'static, MTU>>) -> ! {
    runner.run().await
}
