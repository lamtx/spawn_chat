use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;

use ctrlc::set_handler;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::cloud::Cloud;
use crate::participant::Participant;
use crate::room::{Room, Rooms};

mod cloud;
mod date_serde;
mod http_status_error;
mod participant;
mod room;
mod text_room_event;

pub type AnyError = Box<dyn Error>;

const MAX_CLIENTS: i32 = 1000;
// const INTERVAL_SPAWN_AFTER_MAX_CLIENTS: Duration = Duration::from_secs(1);
pub const URL: &str = "http://127.0.0.1:9339/dev";
pub const SECRET: &str = "c537a335-129f-4cf9-9302-8b5b7f272134";
pub const ROOM: isize = 1;

//noinspection RsUnstableItemUsage
#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let cancellation_token = CancellationToken::new();
    let token = cancellation_token.clone();
    set_handler(move || {
        println!("received Ctrl+C!");
        cancellation_token.cancel();
    })
    .unwrap();

    let cloud = Cloud::new(ROOM);
    let rooms = Rooms::new();
    println!("creating room `{ROOM}`");
    cloud.create(SECRET).await?;
    println!("room `{ROOM}` created");
    for i in 0..MAX_CLIENTS {
        let rooms = rooms.clone();
        let token = token.clone();
        tokio::spawn(async move {
            spawn_client(i, rooms.clone()).await;
            if rooms.size() == (MAX_CLIENTS as usize) {
                println!("spawn more clients");
                spawn_more_clients(token, rooms);
            }
        });
    }
    token.cancelled().await;
    cloud.destroy(SECRET).await.unwrap();
    println!("room destroyed");
    Ok(())
}

async fn spawn_client(i: i32, rooms: Rooms) {
    let participant = Participant {
        username: i.to_string(),
        display: format!("Participant {i}"),
    };
    let room = Room {
        url: String::from(URL),
        room: ROOM,
        me: participant.clone(),
    };
    println!("`{}` joining", room.me.username);
    match room.join().await {
        Ok(joined) => {
            let len = rooms.add(joined);
            println!("`{}` joined (size={len})", participant.username);
        }
        Err(e) => {
            println!("`{}` join failed: {:?}", participant.username, e)
        }
    }
}

fn spawn_more_clients(token: CancellationToken, rooms: Rooms) {
    async fn spawn(rooms: Rooms) {
        let mut i = MAX_CLIENTS;
        loop {
            // tokio::time::sleep(INTERVAL_SPAWN_AFTER_MAX_CLIENTS).await;
            spawn_client(i, rooms.clone()).await;
            if let Some(room) = rooms.remove_first() {
                room.leave();
            }
            i += 1
        }
    }

    tokio::spawn(async move {
        select! {
            _ = spawn(rooms) => {}
            _ = token.cancelled() => {}
        }
    });
}

#[tokio::test]
async fn destroy_room() {
    let cloud = Cloud::new(ROOM);
    cloud.destroy(SECRET).await.unwrap();
    println!("room destroyed");
}
