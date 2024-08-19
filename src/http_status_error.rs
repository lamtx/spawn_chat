use std::error::Error;
use std::fmt::{Display, Formatter};

use hyper::StatusCode;

#[derive(Debug)]
pub struct HttpStatusError {
    pub status_code: StatusCode,
}

impl Display for HttpStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpStatusError ({})", self.status_code)
    }
}

impl Error for HttpStatusError {}