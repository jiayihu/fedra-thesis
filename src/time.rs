use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use hal::prelude::*;
use stm32f4xx_hal as hal;

static TIME: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));

pub fn setup_clocks(cp: &mut cortex_m::Peripherals, rcc: hal::rcc::Rcc) -> hal::rcc::Clocks {
    let syst = &mut cp.SYST;

    // HCLK 42 so that the resulting value is >=24Mhz after division
    let clocks = rcc.cfgr.sysclk(168.mhz()).hclk(42.mhz()).freeze();

    syst.set_reload(SYST::get_ticks_per_10ms() / 10);
    syst.enable_counter();
    syst.enable_interrupt();

    clocks
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let mut time = TIME.borrow(cs).borrow_mut();
        *time += 1;
    })
}

/// Obtain current time in milliseconds
pub fn now() -> u64 {
    cortex_m::interrupt::free(|cs| *TIME.borrow(cs).borrow())
}
