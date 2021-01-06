use std::fmt::Formatter;

#[derive(Debug)]
pub enum BrokerError {
    IOError(std::io::Error),
    K8sError(kube::Error),
    // Provision(String),
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for BrokerError {
    fn from(e: std::io::Error) -> Self {
        BrokerError::IOError(e)
    }
}

impl From<kube::Error> for BrokerError {
    fn from(e: kube::Error) -> Self {
        BrokerError::K8sError(e)
    }
}

pub type Result<T> = std::result::Result<T, BrokerError>;
