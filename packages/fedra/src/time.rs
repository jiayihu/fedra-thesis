use core::cell::{Cell, RefCell};
use cortex_m::interrupt::Mutex;
use hal::{
    stm32::TIM2,
    time::U32Ext,
    timer::{Event as TimerEvent, Timer},
};
use stm32f4xx_hal as hal;

static TIME: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));
static TIMER2: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));

pub fn setup_cycle_counter(cp: &mut rtic::Peripherals) {
    cp.DCB.enable_trace();
    cortex_m::peripheral::DWT::unlock();
    cp.DWT.enable_cycle_counter();
}

pub fn setup_clocks(rcc: hal::rcc::Rcc, tim2: TIM2) -> hal::rcc::Clocks {
    let clocks = rcc.cfgr.sysclk(168.mhz()).hclk(168.mhz()).freeze();

    let mut timer = Timer::tim2(tim2, 1.khz(), clocks);
    timer.listen(TimerEvent::TimeOut);

    cortex_m::interrupt::free(|cs| {
        TIMER2.borrow(cs).replace(Some(timer));
    });

    clocks
}

pub fn handle_tim2_interrupt() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut tim2) = *TIMER2.borrow(cs).borrow_mut() {
            tim2.clear_interrupt(TimerEvent::TimeOut);
        }

        let time = TIME.borrow(cs);
        time.replace(time.get() + 1);
    })
}

/// Obtain current time in milliseconds
pub fn now() -> u64 {
    cortex_m::interrupt::free(|cs| TIME.borrow(cs).get())
}
