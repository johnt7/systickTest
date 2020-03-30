#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

pub use cortex_m::asm::{bkpt, nop};
pub use cortex_m::{iprint, iprintln};
pub use cortex_m_rt::{entry, exception, ExceptionFrame};
pub use cortex_m::peripheral::{self, syst, ITM};
pub use f3::{
    hal::{
        prelude::*,
        delay::Delay,
        prelude,
        stm32f30x::{self, GPIOA, tim6, TIM6},
        gpio,
        gpio::gpioa
    },
    led::Leds,
};

pub fn init() -> (Leds, Button, ITM) {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut systick = cp.SYST;
    let rvr: u32 = 1_000;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(rvr);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

//    let delay = Delay::new(cp.SYST, clocks);

    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let (button, _) = create_lev(dp.GPIOA.split(&mut rcc.ahb));
       
    (leds, button, cp.ITM) 
}

fn create_lev(mut gpa: gpioa::Parts) -> (Button, OutPorts) {
    gpa.pa0.into_floating_input(&mut gpa.moder, &mut gpa.pupdr);
    ( Button {
        idr : 
        unsafe {
            &(*GPIOA::ptr()).idr
        }
    },
    OutPorts{
        pa1:  gpa.pa1.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa2:  gpa.pa2.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa3:  gpa.pa3.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa4:  gpa.pa4.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper)
    })
}

pub struct Button {
    pub idr : &'static stm32f30x::gpioa::IDR
}

impl Button {
    pub fn is_pushed(&self) -> bool {
        self.idr.read().bits() & 0x1 == 1
    }
}

pub struct OutPorts {
    pa1 : gpioa::PA1<gpio::Output<gpio::PushPull>>,
    pa2 : gpioa::PA2<gpio::Output<gpio::PushPull>>,
    pa3 : gpioa::PA3<gpio::Output<gpio::PushPull>>,
    pa4 : gpioa::PA4<gpio::Output<gpio::PushPull>>
}

/*
pub struct Waiter {
    pub tim : &'static  tim6::RegisterBlock
}
impl Waiter {
    pub fn new() -> Waiter {
        Waiter {
            tim : 
            unsafe {
                &*TIM6::ptr()
            }
        }
    }
    pub fn start_timer(&mut self, ms: u16) {
        // Set the timer to go off in `ms` ticks
        // 1 tick = 1 ms
        self.tim.arr.write(|w| w.arr().bits(ms));

        // CEN: Enable the counter
        self.tim.cr1.modify(|_, w| w.cen().set_bit());
    }
    pub fn wait_timer(&mut self) {
        // Wait until the alarm goes off (until the update event occurs)
        while !self.tim.sr.read().uif().bit_is_set() {}

        // Clear the update event flag
        self.tim.sr.modify(|_, w| w.uif().clear_bit());
    }
}
*/