#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

#[allow(unused_imports)]
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
//use embedded_hal::digital::v2::OutputPin; how do I get v2?

pub const RVR: u32 = 9_000;


// PA0 - input from user (blue) button
// PA1, PA2 - signal for one channel
// PA3, PA3 - signal for second channel

pub fn init() -> (Leds, Button, ITM, OutPorts) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

//    let mut flash = dp.FLASH.constrain();
//    let clocks = rcc.cfgr.freeze(&mut flash.acr);
//    let delay = Delay::new(cp.SYST, clocks);

    // set up systick interupt handler
    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(RVR);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let (button, outp) = create_lev(dp.GPIOA.split(&mut rcc.ahb));
       
    (leds, button, cp.ITM, outp) 
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
    pub pa1 : gpioa::PA1<gpio::Output<gpio::PushPull>>,
    pub pa2 : gpioa::PA2<gpio::Output<gpio::PushPull>>,
    pub pa3 : gpioa::PA3<gpio::Output<gpio::PushPull>>,
    pub pa4 : gpioa::PA4<gpio::Output<gpio::PushPull>>
}
