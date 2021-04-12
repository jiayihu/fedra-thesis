use std::sync::atomic::AtomicI8;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use coap::CoAPClient;
use futures::future::join_all;
use tokio::runtime::Builder;

const ADDR: &str = "coap://192.168.1.100:5683/benchmark";

#[tokio::main]
async fn main() {
    let response_times = Arc::new(Mutex::new(Vec::new()));
    let dropped_count = Arc::new(AtomicI8::new(0));

    for _ in 0..15 {
        let request_handles = Arc::new(Mutex::new(Vec::new()));

        let times = response_times.clone();
        let dropped = dropped_count.clone();
        let handles = request_handles.clone();

        thread::spawn(move || {
            let runtime = Arc::new(
                Builder::new_multi_thread()
                    .worker_threads(12)
                    .enable_all()
                    .build()
                    .unwrap(),
            );

            for _ in 0..12 {
                let times = times.clone();
                let dropped = dropped.clone();

                let handle = runtime.spawn_blocking(move || {
                    let timeout = Duration::new(10, 0);
                    println!("Request sent");
                    let instant = Instant::now();
                    let response = CoAPClient::get_with_timeout(ADDR, timeout);

                    match response {
                        Ok(_response) => {
                            let elapsed = instant.elapsed().as_millis();

                            let mut times = times.lock().unwrap();
                            times.push(elapsed);

                            println!("Request received");
                        }
                        Err(_) => {
                            dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            println!("Packet dropped");
                        }
                    }
                });

                handles.lock().unwrap().push(handle);
            }
        })
        .join()
        .unwrap();

        let mut handles = request_handles.lock().unwrap();

        join_all(handles.iter_mut()).await;
    }

    println!("Response times {:?}", response_times.lock().unwrap());
    println!("Dropped packets {:?}", dropped_count);
}
