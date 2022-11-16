#![no_std]
#![no_main]

use cortex_m::prelude::*;
use fugit::ExtU32;

use panic_halt as _;

use rp2040_hal as hal;

use hal::pac;

use embedded_hal::digital::v2::OutputPin;


#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;


const XTAL_FREQ_HZ: u32 = 12_000_000u32;


#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    // let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let _clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();


    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();
        count_down.start(500.millis());
        let _ = nb::block!(count_down.wait());

        led_pin.set_low().unwrap();
        count_down.start(500.millis());
        let _ = nb::block!(count_down.wait());
    }
}

