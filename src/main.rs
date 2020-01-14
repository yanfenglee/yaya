use futures::SinkExt;
use std::sync::mpsc::{Sender,Receiver,channel};
use http::{Request, Response, StatusCode};
use std::{error::Error, fmt, io};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use std::time::{Instant};
mod simple_http;
use simple_http::Http;

extern crate clap;
use clap::{Arg, App};

fn opts() -> (u32, u32, String) {
    let matches = App::new("Simple http benchmark tool")
                            .version("1.0")
                            .author("liyanfeng <yanfeng.li@picahealth.com>")
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
                            .arg(Arg::with_name("url")
                                .help("url to test")
                                .required(true)
                                .index(1))
                            .get_matches();

    let connections: u32 = matches.value_of("connections").unwrap_or("100").parse().unwrap();
    let duration: u32 = matches.value_of("duration").unwrap_or("10").parse().unwrap();
    let url = matches.value_of("url").unwrap_or("localhost:8080").to_string();

    (connections, duration, url)
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let (connections, duration, url) = opts();
    
    let addr = url;


    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();

    for _ in 0..connections {
        let tx = tx.clone();
        let addr = addr.clone();

        tokio::spawn(async move {

            let client = TcpStream::connect(&addr).await.unwrap();
    
            if let Err(e) = process(client, tx).await {
                println!("failed to process connection; error = {}", e);
            }
        });
    }


    let mut sum = 0u32;

    let now = Instant::now();

    loop {
        let elapsed = now.elapsed().as_secs_f64();

        sum += rx.recv().unwrap() as u32;
        if sum % 10000 ==0 {
            println!("finished: {} , speed: {} qps ", sum, (sum as f64/elapsed) as u32);
        }

        if elapsed > duration as f64 {
            break;
        }
    }

    Ok(())
}

async fn process(stream: TcpStream, tx: Sender<u8>) -> Result<(), Box<dyn Error>> {
    
    let mut transport = Framed::new(stream, Http);

    loop {

        let request = Request::get("/")
            .header("Host","localhost:8080")
            .header("User-Agent","wb")
            .body("hello".to_string()).unwrap();

        transport.send(request).await?;

        if let Some(response) = transport.next().await {
            match response {
                Ok(response) => if response.status()==StatusCode::OK {
                    if let Err(_) = tx.send(1) {
                        break;
                    }
                },

                Err(e) => println!("{}", e)
            }
        }
    }

    Ok(())
}
