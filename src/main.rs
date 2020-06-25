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
#[allow(unused_imports)]
use f3jt::{entry, prelude::*, Leds, Button, exception, bkpt, nop, ExceptionFrame, ITM, iprint, iprintln, gpio, gpioa, OutPorts, RVR};
//use f3jt::{entry, Leds, Button, exception, bkpt, nop, ExceptionFrame, ITM, iprint, iprintln, gpio, gpioa, OutPorts, RVR};
use heapless::spsc::Queue;
use heapless::consts::*;
//use cortex_m::singleton;

static mut TOINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut FROMINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut SLEDS: Option<Leds> = None;
static mut SPORTS: Option<OutPorts> = None;
#[entry]
fn main() -> ! {
    /*
    let (toSysSrc, toSysSink) = channel();
    let (frmSysSrc, frmSysSink) = channel();
    */
    let (leds, button, mut itm, outp): (Leds, Button, ITM, OutPorts) = f3jt::init();
    unsafe { SLEDS = Some(leds); }
    unsafe { SPORTS = Some(outp); }
    let mut incoming = unsafe { FROMINT.split().1 };
    let mut outgoing = unsafe { TOINT.split().0 };
    let mut bc = 0;
    let mut count : usize = 0;

    iprintln!(&mut itm.stim[0], "\n\nHello, world! cl={}", RVR);
    loop {
    
        // debounce, loop through a few times to make sure button is properly pushed
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
                },
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
// each signal has two lines, with oposite values 01, 10
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
        outp.pa3.set_high();
        outp.pa4.set_low();
    }
}

/*
*/
const NUMSIGS: u8 = 2;
const STEPS: u8 = 24;
const NUMLEDS: u8 = 8;

struct SigState {
    started: bool,
    start: u8,
    counter: u8
}
impl SigState {
    fn next_step(&mut self, ct: u8) {
        if self.started {
            if self.counter > STEPS/2 {
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
    pub count : u16,
    sig_count: u8,
    which_led: u8,
    prev_led: u8,
    fl: bool,
    sigs: [SigState; NUMSIGS as usize]
}

static mut LSTAT:  SysTickStatic = SysTickStatic {
    count: 0,
    sig_count: 0,
    which_led: 0,
    prev_led: 0,
    fl: false,
    sigs: [SigState {started:false, start: 0, counter: 0}, SigState {started:false, start: 0, counter: 0}]
};


#[exception]
fn SysTick() {

    let mut lstatp = unsafe { &mut LSTAT };
    let mut incoming = unsafe { TOINT.split().1 };
    let mut outgoing = unsafe { FROMINT.split().0 };
    let ledp = unsafe { SLEDS.as_mut().unwrap() };
    let portp = unsafe { SPORTS.as_mut().unwrap() };
/*
    unsafe {
        if let Some(ref mut x) = SLEDS {
            x[lstatp.prev_led as usize].off();
            if lstatp.fl {
                x[lstatp.which_led as usize].on();
            } else {
                x[lstatp.which_led as usize].off();
            }
        }
        write_line(&lstatp.sigs, SPORTS.as_mut().unwrap());
    }
*/
    // flash LED, move if changed
    ledp[lstatp.prev_led as usize].off();
    if lstatp.fl {
        ledp[lstatp.which_led as usize].on();
    } else {
        ledp[lstatp.which_led as usize].off();
    }
    // write output lines
    write_line(&lstatp.sigs, portp);

    for s in &mut lstatp.sigs {
        s.next_step(lstatp.sig_count);
    }

    // send out a tick to main
    lstatp.count += 1;
    if  lstatp.count > 10_000 {
        outgoing.enqueue(lstatp.count as usize).ok().unwrap();
        lstatp.count = 0;

    }
    if incoming.ready() {
        match incoming.dequeue() {
            Some(_) => {
                lstatp.prev_led = lstatp.which_led;
                lstatp.which_led += 1;
                if lstatp.which_led >= NUMLEDS {
                    lstatp.which_led = 0;
                }
            },
            None => panic!()
        }
    }
    
    lstatp.fl = !lstatp.fl;
    lstatp.sig_count = inc_mod(lstatp.sig_count, NUMSIGS);
/*
    let pa1 : gpioa::PA1<gpio::Output<gpio::PushPull>>;
        for i in 1..NUMSIGS {
            if startedp[i] {
                if startp[i] == *ctp {
                    // write hi to port[i]
    //                pa1.odr.write(|w| {
     //                   w.odr1().set_bit();
     //               });
                    startedp[i] = true;
                }
            } else {
                if endp[i] == *ctp {
                    // write low to port[i]
//                    pa1.set_bit();
                    startedp[i] = false;
                }
            }
        }
        *ctp += 1;
        if *ctp >= STEPS {
            *ctp -= STEPS;
    };
     */
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