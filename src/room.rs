use std::collections::LinkedList;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use futures_util::stream::SplitStream;
use reqwest::Client;
use reqwest_websocket::{Error, Message, RequestBuilderExt, WebSocket};
use serde::Serialize;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::participant::Participant;

#[derive(Clone)]
pub struct Rooms {
    rooms: Arc<Mutex<LinkedList<ChatClient>>>,
}

impl Rooms {
    pub fn new() -> Rooms {
        Rooms {
            rooms: Arc::new(Mutex::new(LinkedList::<ChatClient>::new())),
        }
    }

    pub fn size(&self) -> usize {
        self.rooms.lock().unwrap().len()
    }

    pub fn add(&self, client: ChatClient) -> usize {
        let mut lock = self.rooms.lock().unwrap();
        lock.push_back(client);
        lock.len()
    }

    pub fn remove_first(&self) -> Option<ChatClient> {
        self.rooms.lock().unwrap().pop_front()
    }
}
pub struct Room {
    pub url: String,
    pub room: isize,
    pub me: Participant,
}

impl Room {
    pub async fn join(&self) -> Result<ChatClient, Error> {
        let response = Client::default()
            .get(format!("{}/{}/join", &self.url, self.room))
            .query(&[
                ("username", &self.me.username),
                ("display", &self.me.display),
            ])
            .upgrade()
            .send()
            .await?;
        let (mut sink, stream) = response.into_websocket().await?.split();
        let token = CancellationToken::new();
        let me = &self.me.username;
        let cloned_me = me.to_string();
        let cloned_token = token.clone();

        tokio::spawn(async move {
            select! {
                _ = Self::listen(&cloned_me, stream) => {}
                _ = cloned_token.cancelled() => {
                    println!("`{cloned_me}` left");
                }
            }
        });

        let cloned_me = me.to_string();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cloned_cancelled = cancelled.clone();
        tokio::spawn(async move {
            let mut counter = 0i64;
            while !cloned_cancelled.load(Ordering::Relaxed) {
                let rnd = (rand::random::<f64>() + 0.5f64) * 10f64;
                tokio::time::sleep(Duration::from_secs_f64(rnd)).await;
                if cloned_cancelled.load(Ordering::Relaxed) {
                    break;
                }
                counter += 1;
                let message = TextRoomRequest::send_message(format!("{cloned_me} sends {counter}"));
                if sink
                    .send(Message::Text(serde_json::to_string(&message).unwrap()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            println!("`{cloned_me}` stopped")
        });
        return Ok(ChatClient { token, cancelled });
    }

    async fn listen(me: &str, mut stream: SplitStream<WebSocket>) {
        while let Some(_s) = stream.try_next().await.ok().flatten() {
            //    println!("{:?}", _s);
        }
        println!("`{me}` dropped")
    }
}

pub struct ChatClient {
    token: CancellationToken,
    cancelled: Arc<AtomicBool>,
}

impl ChatClient {
    pub fn leave(&self) {
        self.token.cancel();
        self.cancelled.store(true, Ordering::Relaxed)
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
