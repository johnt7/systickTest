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
use aux5::{entry, prelude::*, Delay, Leds, gpioa, GPIOA};

#[entry]
fn main() -> ! {
    let (mut delay, mut leds, mut gpa): (Delay, Leds, gpioa::Parts) = aux5::init();

    // set pa0 as input
    let f = gpa.pa0.into_floating_input(&mut gpa.moder, &mut gpa.pupdr);
    let g = gpa.pa1.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper);

    // PA0 - input from user (blue) button
    // PA1, PA2 - signal for one channel
    // PA3, PA3 - signal for second channel
      
    // implment arduino, with start and length
    // add counter to see how long processing is taking
    // 4 output pins, 2 each for two signals.  
    // each signal has two lines, with oposite values 01, 10

    let mut ms = 100_u8;
    loop {
            delay.delay_ms(ms);
 
        for curr in 0..8 {
            let next = (curr + 1) % 8;

            leds[next].on();
            delay.delay_ms(ms);
            leds[curr].off();
            delay.delay_ms(ms);
            unsafe {
                // A magic address!
//                const GPIOA_IDR: u32 = 0x48000010;
//                let _rd = ptr::read_volatile(GPIOA_IDR as *const u32) & 0x0001;    
                let rd = &(*GPIOA::ptr()).idr.read().bits() & 0x0001;
                if rd == 1 {
                    ms *= 2;
                    while (&(*GPIOA::ptr()).idr.read().bits() & 0x0001) == 1 {};
                }
            }
        }
    }
}
