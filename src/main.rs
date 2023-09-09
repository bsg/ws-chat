use std::{convert::Infallible, path::Path, process::Command, sync::Arc};

use futures::{FutureExt, StreamExt};
use notify::{RecursiveMode, Watcher};
use serde::Serialize;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    RwLock,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    filters::ws::{Message, WebSocket},
    reject::Rejection,
    reply::Reply,
    Filter,
};

pub struct Client {
    pub user_id: usize,
    pub sender: Option<UnboundedSender<Result<Message, warp::Error>>>,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub username: String,
    pub body: String,
}

type FilterResult<T> = Result<T, Rejection>;
type Clients = Arc<RwLock<Vec<Client>>>;

#[tokio::main]
async fn main() {
    println!("Watching chat.ts for changes");
    let mut watcher =
        notify::recommended_watcher(|res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    println!("Recompiling ts... ");
                    Command::new("tsc")
                        .args(["chat.ts", "--outFile", "static/chat.js"])
                        .output()
                        .unwrap();
                }
                _ => (),
            },
            Err(e) => println!("chat.ts watch error: {:?}", e),
        })
        .unwrap();

    watcher
        .watch(Path::new("chat.ts"), RecursiveMode::NonRecursive)
        .unwrap();

    let clients: Clients = Arc::new(RwLock::new(Vec::new()));

    let index = warp::path::end().and(warp::fs::file("static/index.html"));
    let js = warp::path("chat.js").and(warp::fs::file("static/chat.js"));
    let css = warp::path("chat.css").and(warp::fs::file("static/chat.css"));
    let stream = warp::path("stream")
        .and(warp::ws())
        .and(with_clients(clients))
        .and_then(handle_connection);

    let routes = index.or(js).or(css).or(stream);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn handle_connection(ws: warp::ws::Ws, clients: Clients) -> FilterResult<impl Reply> {
    // TODO auth
    Ok(ws.on_upgrade(move |socket| handle_client(socket, clients)))
}

async fn handle_client(ws: WebSocket, clients: Clients) {
    let (tx, mut rx) = ws.split();
    let (client_sender, client_receiver) = unbounded_channel();

    let client_receiver = UnboundedReceiverStream::new(client_receiver);
    tokio::task::spawn(client_receiver.forward(tx).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    clients.write().await.push(Client {
        user_id: 0,
        sender: Some(client_sender),
    });
    println!("Connected");

    while let Some(result) = rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving message");
                break;
            }
        };
        handle_message(msg, clients.clone()).await;
    }

    println!("Disconnected");
}

async fn handle_message(msg: Message, clients: Clients) {
    println!("Received message {:?}", msg);

    if msg.is_text() {
        for client in clients.read().await.iter() {
            if let Some(sender) = &client.sender {
                sender.send(Ok(Message::text(
                    serde_json::to_string(&ChatMessage {
                        username: "".to_string(),
                        body: msg.to_str().unwrap().to_string(),
                    })
                    .unwrap(),
                )));
            }
        }
    }
}
