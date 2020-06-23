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

    iprintln!(&mut itm.stim[0], "\n\nHello, world! cl={}", RVR);

    loop {
    
        // debounce, need to loop through a few times to make sure button is properly pushed
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

fn inc_mod(num: u8, modv: u8) -> u8 {
    let mut res = num + 1;
    if res >= modv {
        res = 0;
    }
    res
}

fn write_line(_started: &[bool], _outp: &mut OutPorts) {
    // write to port for line, high or low based on level
}

/*
*/
const NUMSIGS: u8 = 2;
const _STEPS: u8 = 24;
static NUMLEDS: u8 = 8;

struct SysTickStatic {
    pub count : u16,
    sig_count: u8,
    which_led: u8,
    prev_led: u8,
    fl: bool,
    started: [bool; NUMSIGS as usize],
    start: [u8; NUMSIGS as usize],
    s_count: [u8; NUMSIGS as usize]
}

#[exception]
fn SysTick() {

    let mut incoming = unsafe { TOINT.split().1 };
    let mut outgoing = unsafe { FROMINT.split().0 };

    static mut LSTAT:  SysTickStatic = SysTickStatic {
        count: 0,
        sig_count: 0,
        which_led: 0,
        prev_led: 0,
        fl: false,
        started: [false, false],
        start: [0,12],
        s_count: [12,0]
    };
    let lstatp = unsafe { &mut LSTAT };
    static mut COUNT: u16 = 0;
    static mut SIGCOUNT: u8 = 0;
    static mut WHICHLED: u8 = 0;
    static mut PREVLED: u8 = 0;
    static mut FL: bool = false;
//    static mut sleds: Leds = unsafe { Sleds.take().unwrap() };
//    static mut sleds: Option<Leds> = None;
    static mut STARTED: [bool; NUMSIGS as usize] = [false, false];
    static mut START: [u8; NUMSIGS as usize] = [0,12];
    static mut SCOUNT: [u8; NUMSIGS as usize] = [12,0];

    let ledct = unsafe {&mut WHICHLED};
    let prevct = unsafe {&mut PREVLED};
    let flState = unsafe {&mut FL};
    

    let ctp = unsafe {&mut COUNT};
    let startedp = unsafe {&mut STARTED};
    let startp = unsafe {&mut START};
    let countp = unsafe {&mut SCOUNT};
    let sigcountp = unsafe {&mut SIGCOUNT};

    static mut _PORTS: [u16; NUMSIGS as usize] = [0,0];


    *flState = !*flState;
    unsafe {
        match SLEDS { // https://github.com/rust-lang/rust/issues/28839
            Some(ref mut x) => { 
                x[*prevct as usize].off();
                if *flState {
                    x[*ledct as usize].on();
                } else {
                    x[*ledct as usize].off();
                }
                },
            None => panic!()
        }
        match SPORTS { // https://github.com/rust-lang/rust/issues/28839
            Some(ref mut outps) => { 
                    write_line(&*startedp, outps)
                 },
            None => panic!()
        }
    }
    
    for s in 0..NUMSIGS as usize {
        if startedp[s] {
            if countp[s] > NUMSIGS/2 {
                startedp[s] = false;
                countp[s] = 0;
            } else {
                countp[s] = inc_mod(countp[s], NUMSIGS);
            }
        } else if *sigcountp == startp[s] {
            startedp[s] = true;
            countp[s] = 0;
        }
    }
    *sigcountp = inc_mod(*sigcountp, NUMSIGS);

    // send out a tick to main
    *ctp += 1;
    if *ctp > 10_000 {
        outgoing.enqueue(*ctp as usize).ok().unwrap();
        *ctp = 0;

    }
    if incoming.ready() {
        match incoming.dequeue() {
            Some(_) => {
                *prevct = *ledct;
                *ledct += 1;
                if *ledct >= NUMLEDS {
                    *ledct = 0;
                }
            },
            None => panic!()
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