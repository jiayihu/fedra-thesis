use crate::provider::FedraProvider;
use std::sync::Mutex;

pub struct State {
    pub provider: Mutex<FedraProvider>,
}
