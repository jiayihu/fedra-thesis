use std::thread::sleep;
use std::time::{Duration, UNIX_EPOCH};

fn main() {
    let name_env = std::env::vars().find(|(key, _value)| key.as_str() == "NAME");
    let name = match name_env {
        Some((_key, value)) => value,
        None => "N/A".to_string(),
    };

    let now = std::time::SystemTime::now();
    let now_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis();

    println!("Pod {} boot time {:?}", name, now_ms);

    loop {
        sleep(Duration::new(1000, 0));
    }
}
