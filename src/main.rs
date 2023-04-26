use anyhow::{anyhow, Result};
use humantime::parse_duration;
use std::{
    env,
    io::ErrorKind,
    sync::{mpsc::channel, Arc},
    thread::spawn,
    time::{Duration, SystemTime},
};
use tiny_http::{Response, Server};

fn main() -> Result<()> {
    let started_at: SystemTime = SystemTime::now();

    let listen_address: String =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8000".to_string());

    let started_after = started_at
        + env::var("STARTED_AFTER")
            .map(|value| parse_duration(&value))
            .unwrap_or_else(|_| Ok(Duration::from_secs(0)))
            .expect("Error parsing STARTED_AFTER");

    let ready_after = started_at
        + env::var("READY_AFTER")
            .map(|value| parse_duration(&value))
            .unwrap_or_else(|_| Ok(Duration::from_secs(0)))
            .expect("Error parsing READY_AFTER");

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let server = Arc::new(Server::http(&listen_address).map_err(anyhow::Error::msg)?);

    println!("Starting server on {}", listen_address);

    let server_handle = {
        let server = server.clone();
        spawn(move || {
            loop {
                // blocks until the next request is received
                let request = match server.recv() {
                    Ok(rq) => rq,
                    Err(e) => {
                        if e.kind() == ErrorKind::Other && e.to_string() == "thread unblocked" {
                            break;
                        }
                        println!("error: {}", e);
                        break;
                    }
                };

                let now: SystemTime = SystemTime::now();

                let response = match request.url() {
                    "/started" => {
                        if now >= started_after {
                            Response::from_string("OK").boxed()
                        } else {
                            Response::empty(503).boxed()
                        }
                    }
                    "/ready" => {
                        if now >= ready_after {
                            Response::from_string("OK").boxed()
                        } else {
                            Response::empty(503).boxed()
                        }
                    }
                    _ => Response::from_string("OK").boxed(),
                };
                request.respond(response).expect("this works");
            }
        })
    };

    let _ = rx.recv();
    println!("Stopping");

    server.unblock();

    server_handle
        .join()
        .map_err(|_| anyhow!("error waiting for thread"))?;

    println!("Stopped");
    Ok(())
}
