#![deny(unsafe_code)]
#![no_main]
#![no_std]
/*********
 * TODO
 * - encapsulate button handling so no unsafe
 * - create setTimer/waitTimerClass
 * - create f3-jt crate with button handling and timer
 * - replace aux with specific for system
 * - use systtick instead of main loop
 * ******/
use f3jt::{entry, prelude::*, Leds, Button, exception, bkpt, nop, ExceptionFrame, ITM, iprint, iprintln};
use cortex_m_semihosting::hio;
use core::fmt::Write;

#[entry]
fn main() -> ! {
    let (mut leds, button, mut itm): (Leds, Button, ITM) = f3jt::init();

    // set pa0 as input
//    let f = gpa.pa0.into_floating_input(&mut gpa.moder, &mut gpa.pupdr);
//    let g = gpa.pa1.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper);

    // PA0 - input from user (blue) button
    // PA1, PA2 - signal for one channel
    // PA3, PA3 - signal for second channel
      
    // implment arduino, with start and length
    // add counter to see how long processing is taking
    // 4 output pins, 2 each for two signals.  
    // each signal has two lines, with oposite values 01, 10

//    writeln!(hio::hstdout().unwrap(), "Hello, world!").unwrap();
//      look at getting this working
    iprintln!(&mut itm.stim[0], "Hello, world!");

    loop {
        if button.is_pushed() {
            leds[0].on();
            while button.is_pushed() {};
            leds[0].off();
        }
    }
}

#[exception]
fn SysTick() {
    bkpt;
    static mut FOO: u32 = 0;
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    bkpt;
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    bkpt;
    panic!("Unhandled exception (IRQn = {})", irqn);
}