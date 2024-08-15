use crate::AnyError;
use hyper::StatusCode;
use reqwest::Client;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct Cloud {
    pub id: isize,
    client: Client,
}

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

impl Cloud {
    pub fn new(id: isize) -> Cloud {
        Cloud {
            id,
            client: Client::default(),
        }
    }

    fn with(&self, path: &str) -> String {
        format!("{}/{}/{path}", crate::URL, self.id)
    }

    pub async fn create(&self, secret: String) -> Result<(), AnyError> {
        let resp = self
            .client
            .post(self.with("create"))
            .query(&[
                ("secret", secret.as_ref()),
                ("postTypes", "t"),
                ("post", "https://dev.shoplive.vn/api/ShopLiveTest"),
            ])
            .send()
            .await?;
        if resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(Box::new(HttpStatusError {
                status_code: resp.status(),
            }))
        }
    }
}
