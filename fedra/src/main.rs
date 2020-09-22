#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();

    debug::exit(debug::EXIT_SUCCESS);

    loop {
        hprintln!("Hello, world!").unwrap();
    }
}
