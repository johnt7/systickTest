#![no_main]
#![no_std]
/*********
 * TODO
 * - get timing correct, test
 * - replace deprecated calls
 * - fix led to be one second, independant of RVR
 * ******/
#[allow(unused_imports)]
use f3jt::{
    bkpt, entry, exception, gpio, gpioa, iprint, iprintln, nop, prelude::*, Button, ExceptionFrame,
    Leds, OutPorts, ITM, RVR, SYST, TIM4
};

use heapless::consts::*;
use heapless::spsc::Queue;

static mut TOINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut FROMINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut SLEDS: Option<Leds> = None;
static mut SPORTS: Option<OutPorts> = None;
#[entry]
fn main() -> ! {
    let (leds, button, mut itm, outp, mut systick, tim4): (Leds, Button, ITM, OutPorts, SYST, TIM4) = f3jt::init();
    unsafe {
        SLEDS = Some(leds);
    }
    unsafe {
        SPORTS = Some(outp);
    }
    let mut incoming = unsafe { FROMINT.split().1 };
    let mut outgoing = unsafe { TOINT.split().0 };
    let mut bc = 0;
    let mut count: usize = 0;

    iprintln!(&mut itm.stim[0], "\n\nstart");
    /*
    let mut x = 0;
    for _ in 1..1_000_000 {
        x += 1;
    }
    */
    iprintln!(&mut itm.stim[0], "\n\nstart {}");

    systick.enable_interrupt();

    iprintln!(&mut itm.stim[0], "\n\nHello, world! cl={}", RVR);
    loop {
        let cnt = tim4.cnt.read().bits();
        iprintln!(&mut itm.stim[0], "\n\nclcock! cl={:?}", cnt);
        // debounce, fire on release if button was pushed long enough
        if button.is_pushed() {
            bc += 1;
        } else {
            if bc > 1000 {
                // tell the fast code to increment position
                iprintln!(&mut itm.stim[0], "found push");
                outgoing.enqueue(bc).ok().unwrap();
            } else if bc > 100 {
                iprintln!(&mut itm.stim[0], "almost push")
            }
            bc = 0;
        }

        // check for message back from fast code
        if incoming.ready() {
            match incoming.dequeue() {
                Some(x) => {
                    iprintln!(&mut itm.stim[0], "got back {}", x);
                }
                None => {
                    iprintln!(&mut itm.stim[0], "got none");
                }
            };
        };

        // print a ticker
        count += 1;
        if count >= 100_000 {
            iprintln!(&mut itm.stim[0], "tock");
            count = 0;
        }
    }
}

/// increment a number modulo modv
fn inc_mod(num: u8, modv: u8) -> u8 {
    let mut res = num + 1;
    if res >= modv {
        res = 0;
    }
    res
}

// PA0 - input from user (blue) button
// PA1, PA2 - signal for one channel
// PA3, PA3 - signal for second channel

// 4 output pins, 2 each for two signals.
// each signal has two lines, with opposite values 01, 10
// TODO, change to use current version
#[allow(deprecated)]
fn write_line(sigs: &[SigState], outp: &mut OutPorts) {
    if sigs[0].started {
        outp.pa1.set_high();
        outp.pa2.set_low();
    } else {
        outp.pa1.set_low();
        outp.pa2.set_high();
    }
    if sigs[1].started {
        outp.pa3.set_high();
        outp.pa4.set_low();
    } else {
        outp.pa3.set_low();
        outp.pa4.set_high();
    }
}

/*
*/
const NUMSIGS: u8 = 2;
const STEPS: u8 = 24;
const NUMLEDS: u8 = 8;
const LEDSTEPS: usize = 10_000; // change state once pers second

struct SigState {
    started: bool,
    start: u8,
    counter: u8,
}
impl SigState {
    fn next_step(&mut self, ct: u8) {
        if self.started {
            if self.counter > STEPS / 2 {
                self.started = false;
                self.counter = 0;
            } else {
                self.counter = inc_mod(self.counter, STEPS);
            }
        } else if ct == self.start {
            self.started = true;
            self.counter = 0;
        }
    }
}

struct SysTickStatic {
    pub count: u16,
    sig_count: u8,
    which_led: u8,
    prev_led: u8,
    led_ct: usize,
    fl: bool,
    sigs: [SigState; NUMSIGS as usize],
}

static mut LSTAT: SysTickStatic = SysTickStatic {
    count: 0,
    sig_count: 0,
    which_led: 0,
    prev_led: NUMLEDS - 1,
    led_ct: 0,
    fl: false,
    sigs: [
        SigState {
            started: false,
            start: 0,
            counter: 0,
        },
        SigState {
            started: false,
            start: 0,
            counter: 0,
        },
    ],
};

#[exception]
fn SysTick() {
    let mut lstatp = unsafe { &mut LSTAT };
    let mut incoming = unsafe { TOINT.split().1 };
    let mut outgoing = unsafe { FROMINT.split().0 };
    let ledp = unsafe { SLEDS.as_mut().unwrap() };
    let portp = unsafe { SPORTS.as_mut().unwrap() };

    // flash LED, move if got button push
    lstatp.led_ct += 1;
    if lstatp.led_ct > LEDSTEPS {
        outgoing.enqueue(lstatp.led_ct as usize + 11).ok().unwrap();
        ledp[lstatp.prev_led as usize].off();
        if lstatp.fl {
            ledp[lstatp.which_led as usize].on();
        } else {
            ledp[lstatp.which_led as usize].off();
        }
        lstatp.led_ct = 0;
        lstatp.fl = !lstatp.fl;
    }

    // write output lines
    write_line(&lstatp.sigs, portp);
    // update the signal positions
    for s in &mut lstatp.sigs {
        s.next_step(lstatp.sig_count);
    }
    lstatp.sig_count = inc_mod(lstatp.sig_count, STEPS);

    /*
        // send out a tick to main
        lstatp.count += 1;
        if  lstatp.count > LEDSTEPS as u16 {
    //        outgoing.enqueue(lstatp.count as usize).ok().unwrap();
            lstatp.count = 0;

        }
        */

    // see if we have a change signal
    if incoming.ready() {
        match incoming.dequeue() {
            Some(_) => {
                lstatp.prev_led = lstatp.which_led;
                lstatp.which_led = inc_mod(lstatp.which_led, NUMLEDS);
                // todo - update the signals
            }
            None => panic!(),
        }
    }
}

#[allow(path_statements)]
#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    bkpt;
    panic!("{:#?}", ef);
}

#[allow(path_statements)]
#[exception]
fn DefaultHandler(irqn: i16) {
    #[allow(path_statements)]
    bkpt;
    panic!("Unhandled exception (IRQn = {})", irqn);
}

// current code 37.1 hz
