use anyhow::Result;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::hyper::body::Bytes;
use warp::hyper::{Response, StatusCode};
use warp::{Filter, Reply};

type State = Arc<Mutex<sled::Db>>;

async fn handle_health() -> Result<impl Reply, Infallible> {
    Ok("Healthy")
}

async fn get_record(key: String, state: State) -> Result<impl Reply, Infallible> {
    let key = key.as_bytes();
    let db = state.lock().unwrap();

    match db.get(key) {
        Ok(result) => match result {
            Some(value) => {
                let response = Response::builder().body(value.as_ref().to_vec()).unwrap();

                Ok(response)
            }
            None => {
                let response = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(vec![])
                    .unwrap();

                Ok(response)
            }
        },
        Err(e) => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("{}", e).into_bytes())
                .unwrap();

            Ok(response)
        }
    }
}

async fn post_record(key: String, value: Bytes, state: State) -> Result<impl Reply, Infallible> {
    let db = state.lock().unwrap();
    let key = key.as_bytes();

    match db.insert(key, value.as_ref()) {
        Ok(_result) => {
            let response = Response::builder().body("".into()).unwrap();

            Ok(response)
        }
        Err(e) => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("{}", e).into_bytes())
                .unwrap();

            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let db = sled::open("db")?;
    let state = Arc::new(Mutex::new(db));

    let healthz = warp::path("healthz").and_then(|| handle_health());
    let get = warp::path!("record" / String)
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(|key: String, state: State| get_record(key, state));
    let insert = warp::path!("record" / String)
        .and(warp::post())
        .and(warp::body::bytes())
        .and(with_state(state.clone()))
        .and_then(|key: String, record: Bytes, state: State| post_record(key, record, state));

    let routes = healthz.or(get).or(insert).with(warp::log("api"));

    warp::serve(routes).run(([0, 0, 0, 0], 8084)).await;

    Ok(())
}

fn with_state(state: State) -> impl warp::Filter<Extract = (State,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}
