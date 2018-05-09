#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate metro_m0 as hal;
#[cfg(not(feature = "use_semihosting"))]
extern crate panic_abort;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;

#[cfg(feature = "use_semihosting")]
macro_rules! dbgprint {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hio;
            use core::fmt::Write;
            let mut stdout = hio::hstdout().unwrap();
            writeln!(stdout, $($arg)*).ok();
        }
    };
}

#[cfg(not(feature = "use_semihosting"))]
macro_rules! dbgprint {
    ($($arg:tt)*) => {{}};
}

use hal::clock::GenericClockController;
use hal::prelude::*;
use rtfm::{app, Threshold};

app! {
    device: hal::atsamd21g18a,

    resources: {
        static RED_LED: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>;
        static TIMER: hal::timer::TimerCounter3;
    },

    tasks: {
        TC3: {
            path: timer,
            resources: [TIMER, RED_LED],
        },
    }
}

fn timer(_t: &mut Threshold, mut r: TC3::Resources) {
    if r.TIMER.wait().is_ok() {
        r.RED_LED.toggle();
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn init(mut p: init::Peripherals /* , r: init::Resources */) -> init::LateResources {
    let interval = 1.hz();

    let mut clocks = GenericClockController::new(
        p.device.GCLK,
        &mut p.device.PM,
        &mut p.device.SYSCTRL,
        &mut p.device.NVMCTRL,
    );
    let gclk0 = clocks.gclk0();
    let mut pins = p.device.PORT.split();

    let mut tc3 = hal::timer::TimerCounter::tc3_(
        &clocks.tcc2_tc3(&gclk0).unwrap(),
        p.device.TC3,
        &mut p.device.PM,
    );
    dbgprint!("start timer");
    tc3.start(interval);
    tc3.enable_interrupt();

    dbgprint!("done init");
    init::LateResources {
        RED_LED: pins.pa17.into_open_drain_output(&mut pins.port),
        TIMER: tc3,
    }
}
