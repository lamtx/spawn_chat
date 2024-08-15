use std::time::Duration;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use reqwest::Client;
use reqwest_websocket::{Error, Message, RequestBuilderExt};
use serde::Serialize;

use crate::participant::Participant;

pub struct Room {
    pub url: String,
    pub room: isize,
    pub me: Participant,
}

impl Room {
    pub async fn join(&self) -> Result<(), Error> {
        let response = Client::default()
            .get(format!("{}/{}/join", &self.url, self.room))
            .query(&[
                ("username", &self.me.username),
                ("display", &self.me.display),
            ])
            .upgrade()
            .send()
            .await?;
        let (mut sink, mut stream) = response.into_websocket().await?.split();
        let me = &self.me.username;
        println!("`{me}` joined");
        let cloned_me = me.to_string();
        tokio::spawn(async move {
            while let Some(_s) = stream.try_next().await.ok().flatten() {
                println!("{:?}", _s);
            }
            println!("`{cloned_me}` dropped")
        });
        let mut counter = 0i64;
        loop {
            let rnd = (rand::random::<f64>() + 0.5f64) * 10f64;
            tokio::time::sleep(Duration::from_secs_f64(rnd)).await;
            counter += 1;
            let message = TextRoomRequest::send_message(format!("{me} sends {counter}"));
            if sink
                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                .await
                .is_err()
            {
                break Ok(());
            }
            if sink
                .send(Message::Ping(const { Vec::new() }))
                .await
                .is_err()
            {
                break Ok(());
            }
        }
    }
}

#[derive(Serialize)]
struct TextRoomRequest {
    textroom: &'static str, // message
    r#type: &'static str,   // t
    text: String,
}

impl TextRoomRequest {
    pub fn send_message(text: String) -> TextRoomRequest {
        TextRoomRequest {
            textroom: "message",
            r#type: "t",
            text,
        }
    }
}
