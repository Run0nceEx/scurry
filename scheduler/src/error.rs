use tokio::time::Error as TimeError;

#[derive(Debug)]
pub enum Error {
    TimeCacheError(TimeError),

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl From<TimeError> for Error {
    fn from(x: TimeError) -> Self {
        Self::TimeCacheError(x)
    }
}

impl std::error::Error for Error {}
