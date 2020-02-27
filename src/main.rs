//use futures::SinkExt;
use std::sync::mpsc::{Sender,Receiver,channel};
use std::{error::Error};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed};
use std::time::{Instant};
use yaya::simple_http::Http;
use yaya::{Payload, ProcStatus};
use std::sync::{Arc, RwLock};

use hyper::{Client, Request, Body};

extern crate clap;
use clap::{Arg, App};

// use std::sync::atomic::{AtomicUsize, Ordering};
// static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(1);

fn opts() -> (u32, u32, String, String, String) {
    let matches = App::new("Simple http benchmark tool")
                            .version("1.0")
                            .author("liyanfeng <muxsdt@gmail.com>")
                            .arg(Arg::with_name("connections")
                                .short("c")
                                .long("connections")
                                .help("Connections to keep open")
                                .takes_value(true))
                            .arg(Arg::with_name("duration")
                                .short("d")
                                .long("duration")
                                .help("Duration(seconds) of test")
                                .takes_value(true))
                            .arg(Arg::with_name("method")
                                .short("X")
                                .long("method")
                                .help("http method: GET,POST,DELETE,PUT...")
                                .takes_value(true))
                            .arg(Arg::with_name("body")
                                .short("b")
                                .long("body")
                                .help("http body")
                                .takes_value(true))
                            .arg(Arg::with_name("url")
                                .help("url to test")
                                .required(true)
                                .index(1))
                            .get_matches();

    let connections: u32 = matches.value_of("connections").unwrap_or("100").parse().unwrap();
    let duration: u32 = matches.value_of("duration").unwrap_or("10").parse().unwrap();
    let method: String = matches.value_of("method").unwrap_or("GET").parse().unwrap();
    let body: String = matches.value_of("body").unwrap_or("{}").parse().unwrap();
    let url = matches.value_of("url").unwrap().to_string();

    (connections, duration, method, body, url)
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let (connections, duration, method, body, urlstr) = opts();


    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();

    let status = Arc::new(RwLock::new(ProcStatus::RUNNING));

    // do benchmarking
    for _i in 0..connections {
        let tx = tx.clone();
        let addr = payload.host.clone();
        let payload = payload.clone();
        let status = status.clone();

        tokio::spawn(async move {

            let client = Client::new();

            loop {
                let tx = tx.clone();
                let status = status.clone();
                if *status.read().unwrap() == ProcStatus::TERMINATE {
                    let _ = tx.send(2);
                    break;
                }

                let req = Request::builder()
                    .method("GET")
                    .uri("http://localhost:3000/")
                    .body(Body::from("Hallo"))
                    .expect("request build");

                let response = client.request(req).await.expect("request error");
                if response.status().is_success() {
                    tx.send(1).expect("send finish error");
                }
            }
            
        });
    }


    let mut sum = 1;
    let now = Instant::now();

    // calc performance
    while *status.read().unwrap() == ProcStatus::RUNNING {
        let elapsed = now.elapsed().as_secs_f64();

        if let Ok(n) = rx.try_recv() {
            sum += n as u32;
        }

        if sum % 10000 == 0 {
            println!("finished: {} , average speed: {} qps ", sum, (sum as f64/elapsed) as u32);
        }

        if elapsed > duration as f64 {
            println!("finished: {} , average speed: {} qps ", sum, (sum as f64/elapsed) as u32);
            println!("time over! see last result");
            *status.write().unwrap() = ProcStatus::TERMINATE;
        }

        tokio::task::yield_now().await;

    }

    // wait proc terminate
    let mut finished = 0;
    while finished != connections {
        if rx.recv().unwrap() == 2 {
            finished += 1;
        }
    }

    println!("all finished!!!");

    Ok(())
}
