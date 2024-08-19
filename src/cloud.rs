use hyper::StatusCode;
use reqwest::Client;

use crate::AnyError;
use crate::http_status_error::HttpStatusError;

pub struct Cloud {
    pub id: isize,
    client: Client,
}

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

    pub async fn create(&self, secret: &str) -> Result<(), AnyError> {
        let resp = self
            .client
            .post(self.with("create"))
            .query(&[
                ("secret", secret),
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

    pub async fn destroy(&self, secret: &str) -> Result<(), AnyError> {
        let resp = self
            .client
            .post(self.with("destroy"))
            .query(&[("secret", secret)])
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
