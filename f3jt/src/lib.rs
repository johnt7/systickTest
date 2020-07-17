#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

#[allow(unused_imports)]
pub use cortex_m::asm::{bkpt, nop};
pub use cortex_m::{iprint, iprintln};
pub use cortex_m_rt::{entry, exception, ExceptionFrame};
pub use cortex_m::peripheral::{self, syst, ITM, SYST};
pub use f3::{
    hal::{
        prelude::*,
        delay::Delay,
        prelude,
        stm32f30x::{self, GPIOA, tim6, TIM6, TIM4},
        gpio,
        gpio::gpioa
    },
    led::Leds,
};
//use embedded_hal::digital::v2::OutputPin; how do I get v2?

pub const RVR: u32 = 9_000; 
//pub const RVR: u32 = 9_000;
//pub const RVR: u32 = 9; // 1mhz 40khz, 24 samples


// PA0 - input from user (blue) button
// PA1, PA2 - signal for one channel
// PA3, PA3 - signal for second channel

pub fn init() -> (Leds, Button, ITM, OutPorts, SYST, TIM4) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();
    let mut itm = cp.ITM;
    iprintln!(&mut itm.stim[0], "\n\nin init! cl={}", RVR);
    /*
    {
        let &mut foo = &mut dp.RCC.apb1; gives RCC (not Rcc like contrain)
    }
    */
    unsafe { dp.RCC.apb1enr.modify(|r,w| w.bits(r.bits()|8)) };
    let mut rcc = dp.RCC.constrain();
    //rcc.apb1.enr().modify(|r,w| w.TIM4ENR);
    //let foo = rcc.apb1.enr().modify(|_, w| w.tim4en.set_bit())

//    let mut flash = dp.FLASH.constrain();
//    let clocks = rcc.cfgr.freeze(&mut flash.acr);
//    let delay = Delay::new(cp.SYST, clocks);
    let tim4 = dp.TIM4;
    unsafe { tim4.arr.write(|w| w.bits(0xffffffff)) };
//    unsafe { tim4.cr1.write(|w| w.bits(0)) };
    unsafe { tim4.psc.write(|w| w.bits(0)) };
    unsafe { tim4.cr1.write(|w| w.bits(1)) }; // set counter up and enable


    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let (button, outp) = create_lev(dp.GPIOA.split(&mut rcc.ahb));

// set up systick interupt handler
    let mut systick = cp.SYST;
    systick.set_clock_source(syst::SystClkSource::Core);
    iprintln!(&mut itm.stim[0], "\n\nin init 1");
    systick.set_reload(RVR);
    iprintln!(&mut itm.stim[0], "\n\nin init2");
    systick.clear_current();
    systick.enable_counter();
//    systick.enable_interrupt();

    (leds, button, itm, outp, systick, tim4) 
}

fn create_lev(mut gpa: gpioa::Parts) -> (Button, OutPorts) {
    gpa.pa0.into_floating_input(&mut gpa.moder, &mut gpa.pupdr);
    ( Button {
        idr : 
        unsafe {
            &(*GPIOA::ptr()).idr
        },
        ct : 0
    },
    OutPorts{
        pa1:  gpa.pa1.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa2:  gpa.pa2.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa3:  gpa.pa3.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper),
        pa4:  gpa.pa4.into_push_pull_output(&mut gpa.moder, &mut gpa.otyper)
    })
}

pub struct Button {
    pub idr : &'static stm32f30x::gpioa::IDR,
    pub ct : usize
}

impl Button {
    pub fn is_pushed(&self) -> bool {
        self.idr.read().bits() & 0x1 == 1
    }
    pub fn push_rel(&mut self) -> bool {
        let mut res : bool = false;
        if self.is_pushed() {
            self.ct += 1;
        } else {
            if self.ct > 1000 {
                res = true;
            }
            self.ct = 0;
        };
        res
    }
}

pub struct OutPorts {
    pub pa1 : gpioa::PA1<gpio::Output<gpio::PushPull>>,
    pub pa2 : gpioa::PA2<gpio::Output<gpio::PushPull>>,
    pub pa3 : gpioa::PA3<gpio::Output<gpio::PushPull>>,
    pub pa4 : gpioa::PA4<gpio::Output<gpio::PushPull>>
}
