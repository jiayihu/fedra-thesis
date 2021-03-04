use log::{Level, Metadata, Record};
use rtt_target::{rprintln, rtt_init_print};

struct RttLogger {}

impl log::Log for RttLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        //
        if let Some(module) = record.module_path() {
            if let Some(line) = record.line() {
                let is_unrecognised = line == 559 || line == 560;
                let is_non_unicast = line == 952;

                if module.contains("smoltcp::iface::ethernet")
                    && (is_unrecognised || is_non_unicast)
                {
                    return;
                }
            }
        }

        rprintln!(
            "{}:{} -- {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static mut LOGGER: RttLogger = RttLogger {};

pub fn init_logger() {
    rtt_init_print!();

    unsafe {
        log::set_logger(&LOGGER).unwrap();
    }

    log::set_max_level(log::LevelFilter::Debug);
}
