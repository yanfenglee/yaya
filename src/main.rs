use futures::SinkExt;
use std::sync::mpsc::{Sender,Receiver,channel};
use http::{Request, StatusCode};
use std::{error::Error};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed};
use std::time::{Instant};
use yaya::simple_http::Http;
use yaya::Payload;
use url::Url;

extern crate clap;
use clap::{Arg, App};

fn opts() -> (u32, u32, String, String, String) {
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

    let url = match Url::parse(urlstr.as_str()) {
        Ok(u) => u,
        Err(_) => {
            println!("url error, example: http://localhost:8080/");
            return Ok(());
        }
    };

    let port = match url.port() {
        Some(p) => format!(":{}",p),
        None => String::from(""),
    };

    let host = match url.host_str() {
        Some(h) => String::from(h),
        None => {
            println!("url error, e.g: http://localhost:8080/");
            return Ok(());
        }
    };

    let payload = Payload {
        host: host + port.as_str(),
        path: String::from(url.path()),
        method: method,
        body: body,
    };

    println!("{:?}", payload);


    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();

    for _ in 0..connections {
        let tx = tx.clone();
        let addr = payload.host.clone();
        let payload = payload.clone();

        tokio::spawn(async move {

            let client = TcpStream::connect(addr).await.unwrap();
    
            if let Err(e) = process(client, &payload, tx).await {
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

async fn process(stream: TcpStream, payload: &Payload, tx: Sender<u8>) -> Result<(), Box<dyn Error>> {
    
    let mut transport = Framed::new(stream, Http);

    loop {

        let request = Request::get(payload.path.clone())
            .header("Host", payload.host.clone())
            .header("Content-Type", "application/json")
            .body(payload.body.clone()).unwrap();

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
