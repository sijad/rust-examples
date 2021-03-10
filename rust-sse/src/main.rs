use std::sync::{Arc, Mutex, mpsc};
use tide::Request;
use tide::sse;
use tide::prelude::{Deserialize};

#[derive(Debug, Deserialize)]
struct Message {
    text: String,
}

#[derive(Clone)]
struct State {
    messages_txs: Arc<Mutex<Vec<mpsc::Sender<String>>>>
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let mut app = tide::with_state(State {
        messages_txs: Arc::new(Mutex::new(Vec::new()))
    });
    app.at("/message").post(post_message);
    app.at("/sse").get(sse::endpoint(sse_handler));
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn post_message(mut req: Request<State>) -> tide::Result {
    let Message { text } = req.body_json().await?;
    tide::log::info!("ok so far");
    req.state().messages_txs.lock().unwrap().retain(|tx| {
        match tx.send(text.clone()) {
            Ok(()) => {
                true
            },
            Err(err) => { 
                tide::log::error!("{}", err);
                false
            }
        }
    });
    Ok("ok".into())
}

async fn sse_handler(req: Request<State>, sender: sse::Sender) -> tide::Result<()> {
    let (tx, rx) = mpsc::channel::<String>();
    req.state().messages_txs.lock().unwrap().push(tx);
    loop {
        let rcv = rx.recv();
        match rcv {
            Ok(msg) => {
                if let Err(_) = sender.send("message", msg, None).await {
                    break;
                }
            },
            Err(_) => {
                break;
            },
        }
    };
    Ok(())
}
