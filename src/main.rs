#![no_main]
#![no_std]
#![allow(unused_variables, unused_imports, unused_must_use)]


use cortex_m::prelude::*;
use fugit::ExtU32;
use fugit::RateExtU32;
use hal::Timer;
use hal::timer::Instant;
use rp2040_hal::gpio::Interrupt::EdgeLow;
use embedded_hal::digital::v2::ToggleableOutputPin;
use panic_halt as _;

use rp2040_hal as hal;

use hal::{pac, uart::{DataBits, UartConfig, StopBits}};

use embedded_hal::digital::v2::OutputPin;
use rp2040_hal::Clock;
use core::fmt::Write;

use hal::pac::interrupt;

use core::cell::RefCell;
use critical_section::Mutex;

type LedPin = hal::gpio::Pin<hal::gpio::bank0::Gpio25, hal::gpio::PushPullOutput>;
type ButtonPin = hal::gpio::Pin<hal::gpio::bank0::Gpio16, hal::gpio::PullUpInput>;

type LedAndButton = (LedPin, ButtonPin);

static GLOBAL_PINS: Mutex<RefCell<Option<LedAndButton>>> = Mutex::new(RefCell::new(None));



#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;


const XTAL_FREQ_HZ: u32 = 12_000_000u32;


#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
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
    let count_down = timer.count_down();

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );



    let led = pins.gpio25.into_mode::<hal::gpio::PushPullOutput>();
    let in_pin = pins.gpio16.into_mode();
    
    in_pin.set_interrupt_enabled(EdgeLow, true);

    critical_section::with(|cs| {
        GLOBAL_PINS.borrow(cs).replace(Some((led, in_pin)));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }



    let uart_pins = (
        pins.gpio12.into_mode::<hal::gpio::FunctionUart>(),
        pins.gpio13.into_mode::<hal::gpio::FunctionUart>()
    );

    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

        

    loop {

        let inst  = timer.get_counter().ticks();
        if  inst % 1000000 == 0 {
            
            critical_section::with(|cs| {
                writeln!(uart, "happneed, inst {} , inst % 1000000 {}", inst, inst % 10000);
                if let Some((led, in_pin)) = GLOBAL_PINS.borrow(cs).borrow_mut().as_mut() {
                    led.toggle();
                } else {
                    // taken
                    let x = 1;
                }
            });
        }



    }
}


#[interrupt]
fn IO_IRQ_BANK0() {
    critical_section::with(|cs| {
        let a = GLOBAL_PINS.borrow(cs);
        let mut b = a.borrow_mut();
        let c = b.as_mut();

        if let Some(d) = c {
            let (e, f) = d;
            if f.interrupt_status(EdgeLow) {
                e.toggle();
            }


            f.clear_interrupt(EdgeLow);

        } else {
            // it was empty
            let x = 1;

        }
    });
}