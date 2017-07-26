//! Blinky using futures

#![allow(unreachable_code)] // for the `try_nb!` macro
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
use blue_pill::{Timer, stm32f103xx};
use futures::future::{self, Loop};
use futures::{Async, Future};
use hal::prelude::*;
use rtfm::{P0, T0, TMax};

// CONFIGURATION
const FREQUENCY: Hertz = Hertz(1);

// RESOURCES
peripherals!(stm32f103xx, {
    GPIOC: Peripheral {
        ceiling: C0,
    },
    RCC: Peripheral {
        ceiling: C0,
    },
    TIM3: Peripheral {
        ceiling: C0,
    },
});

// INITIALIZATION PHASE
fn init(ref prio: P0, thr: &TMax) {
    let gpioc = &GPIOC.access(prio, thr);
    let rcc = &RCC.access(prio, thr);
    let tim3 = TIM3.access(prio, thr);

    let timer = Timer(&*tim3);

    led::init(gpioc, rcc);
    timer.init(FREQUENCY.invert(), rcc);
}

// IDLE LOOP
fn idle(ref prio: P0, ref thr: T0) -> ! {
    let tim3 = TIM3.access(prio, thr);

    let timer = Timer(&*tim3);

    // Tasks
    let mut blinky = future::loop_fn::<_, (), _, _>(true, |state| {
        future::poll_fn(move || Ok(Async::Ready(try_nb!(timer.wait()))))
            .map(move |_| {
                if state {
                    Green.on();
                } else {
                    Green.off();
                }

                Loop::Continue(!state)
            })
    });

    // Event loop
    timer.resume();
    loop {
        blinky.poll().unwrap(); // NOTE(unwrap) E = !
    }
}

// TASKS
tasks!(stm32f103xx, {});
