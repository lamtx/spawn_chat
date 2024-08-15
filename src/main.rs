use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::AtomicU16;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::participant::Participant;
use crate::room::Room;

mod cloud;
mod date_serde;
mod participant;
mod room;
mod text_room_event;

pub type AnyError = Box<dyn Error>;

const MAX_CLIENTS: i32 = 1000;
pub const URL: &str = "http://127.0.0.1:9339/dev";

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let room_id = 12458;
    let secret = Uuid::new_v4().to_string();
    let cloud = Cloud::new(room_id);
    println!("creating room `{room_id}`");
    cloud.create(secret).await?;
    println!("room `{room_id}` created");
    for i in 0..MAX_CLIENTS {
        tokio::spawn(async move {
            let participant = Participant {
                username: i.to_string(),
                display: format!("Participant {i}"),
            };
            let room = Room {
                url: String::from(URL),
                room: room_id,
                me: participant,
            };
            println!("`{}` joining", room.me.username);
            match room.join().await {
                Ok(_) => println!("`{}` left", room.me.username),
                Err(e) => println!("`{}` join failed: {:?}", room.me.username, e),
            };
        });
    }

    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(10000000)).await;
        }
    })
    .await
    .unwrap();
    println!("spawn clients complete");

    Ok(())
}
