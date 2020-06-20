//#![deny(unsafe_code)]
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
use f3jt::{entry, prelude::*, Leds, Button, exception, bkpt, nop, ExceptionFrame, ITM, iprint, iprintln, gpio, gpioa};
use heapless::spsc::Queue;
use heapless::consts::*;
//use cortex_m::singleton;

static mut TOINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut FROMINT: Queue<usize, U4> = Queue(heapless::i::Queue::new());
static mut SLEDS: Option<Leds> = None;
#[entry]
fn main() -> ! {
    /*
    let (toSysSrc, toSysSink) = channel();
    let (frmSysSrc, frmSysSink) = channel();
    */
    let (mut leds, button, mut itm): (Leds, Button, ITM) = f3jt::init();
    unsafe { SLEDS = Some(leds); }
    let mut incoming = unsafe { FROMINT.split().1 };
    let mut outgoing = unsafe { TOINT.split().0 };
    let mut bc = 0;
//    let mut outtest = unsafe { fromInt.split().0 };

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

    // signals to and from systick
    // talk to ports in systick

    iprintln!(&mut itm.stim[0], "\n\nHello, world!");

    let mut count : usize = 0;
    loop {
        iprintln!(&mut itm.stim[0], "top of loop");
//        outgoing.enqueue(count).ok().unwrap();
        iprintln!(&mut itm.stim[0], "enqueued ct={}", count);
    
        if button.is_pushed() {
//            iprintln!(&mut itm.stim[0], "pushed");
            //leds[0].on();
            bc += 1;
             //leds[0].off();
//            iprintln!(&mut itm.stim[0], "released");
        } else {
            if bc > 100 {
                iprintln!(&mut itm.stim[0], "found push");
                outgoing.enqueue(bc).ok().unwrap();
            }
            bc = 0;
        }
/*
        while !incoming.ready() {};

        match incoming.dequeue() {
            Some(x) => {
                iprintln!(&mut itm.stim[0], "got back {}", x);
            },
            None => {
                iprintln!(&mut itm.stim[0], "x");
            }
        };
*/
        count += 1;
    }
}

const NUMSIGS: usize = 2;
const _STEPS: u16 = 24;
#[exception]
fn SysTick() {
    /*
    static mut initialized: bool = false;
    if !*initialized {
        unsafe { *initialized = true;}
        let x: &'static mut u16 = singleton!(: u16 = 0).unwrap();
    }
    x += 1;
*/
    let mut incoming = unsafe { TOINT.split().1 };
    let mut outgoing = unsafe { FROMINT.split().0 };

    static mut COUNT: u16 = 0;
    static mut WHICHLED: u8 = 0;
    static mut PREVLED: u8 = 0;
//    static mut sleds: Leds = unsafe { Sleds.take().unwrap() };
//    static mut sleds: Option<Leds> = None;
    static mut STARTED: [bool; NUMSIGS] = [false, false];
    static mut START: [u16; NUMSIGS] = [0,12];
    static mut END: [u16; NUMSIGS] = [12,0];
    static mut _PORTS: [u16; NUMSIGS] = [0,0];
/*
    unsafe {
        if let Some(x) = Sleds.take() {
                sleds = Some(x);
        }
    }
*/  
    let ctp = unsafe {&mut COUNT};
    let ledct = unsafe {&mut WHICHLED};
    let prevct = unsafe {&mut PREVLED};
    *ctp += 1;
    

    let ctp = unsafe {&mut COUNT};
    let _startedp = unsafe {&mut STARTED};
    let _startp = unsafe {&mut START};
    let _endp = unsafe {&mut END};
    unsafe {
            match SLEDS { // https://github.com/rust-lang/rust/issues/28839
                Some(ref mut x) => { 
                    x[*prevct as usize].off();
                    x[*ledct as usize].on();
//                    let mut x1 = &mut x[1];

//                    x1.on();
                 },
                None => panic!()

        }
    }

//    outgoing.enqueue(*ctp as usize).ok().unwrap();
    *ctp += 1;
    if incoming.ready() {
        match incoming.dequeue() {
            Some(x) => {
                outgoing.enqueue(x + *ctp as usize).ok().unwrap();
                *ctp = 0;
            },
            None => {
                outgoing.enqueue(0).ok().unwrap();
            }
        }
    }
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