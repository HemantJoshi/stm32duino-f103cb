//! Two concurrent tasks using `await!`

#![allow(unreachable_code)] // for the `await!` macro
#![deny(warnings)]
#![feature(const_fn)]
#![feature(used)]
#![no_std]

extern crate blue_pill;

extern crate embedded_hal as hal;

// version = "0.2.3"
extern crate cortex_m_rt;

// version = "0.1.0"
#[macro_use]
extern crate cortex_m_rtfm as rtfm;

extern crate futures;

#[macro_use]
extern crate nb;

use blue_pill::led::{self, Green};
use blue_pill::time::Hertz;
use blue_pill::{Serial, Timer, stm32f103xx};
use futures::future::{self, Loop};
use futures::{Async, Future};
use hal::prelude::*;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const BAUD_RATE: Hertz = Hertz(115_200);
const FREQUENCY: Hertz = Hertz(1);

// RESOURCES
peripherals!(stm32f103xx, {
    AFIO: Peripheral {
        ceiling: C0,
    },
    GPIOA: Peripheral {
        ceiling: C0,
    },
    GPIOC: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM3: Peripheral {
        ceiling: C0,
    },
    USART1: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let afio = &AFIO.access(prio, thr);
    let gpioa = &GPIOA.access(prio, thr);
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim3 = TIM3.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let timer = Timer(&*tim3);
    let serial = Serial(&*usart1);

    led::init(gpioc, rcc);

    serial.init(BAUD_RATE.invert(), afio, None, gpioa, rcc);

    timer.init(FREQUENCY.invert(), rcc);
}

// IDLE LOOP
#[inline(never)]
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let tim3 = TIM3.access(prio, thr);
    let usart1 = USART1.access(prio, thr);

    let timer = Timer(&*tim3);
    let serial = Serial(&*usart1);

    // Tasks
    let mut blinky = (|| {
        let mut state = false;
        loop {
            await!(timer.wait()).unwrap(); // NOTE(unwrap) E = !

            state != state;

            if state {
                Green.on();
            } else {
                Green.off();
            }
        }
    })();

    let mut loopback = (|| loop {
                            let byte = await!(serial.read()).unwrap();
                            await!(serial.write()).unwrap();
                        })();

    // Resume the timer count
    timer.resume();

    // Event loop
    loop {
        blinky.resume();
        loopback.resume();
    }
}

// TASKS
tasks!(stm32f103xx, {});
