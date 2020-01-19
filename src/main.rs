use futures::SinkExt;
use std::sync::mpsc::{Sender,Receiver,channel};
use http::{StatusCode, Method};
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

//use std::sync::atomic::{AtomicUsize, Ordering};
//static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

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

    for i in 0..connections {
        let tx = tx.clone();
        let addr = payload.host.clone();
        let payload = payload.clone();

        tokio::spawn(async move {

            loop {
                //println!("new connectoin begin-----{}-------------", i);
                let tx = tx.clone();
                match TcpStream::connect(&addr).await {
                    Ok(client) => {
                        //println!("new connectoin------{}------------", i);
                        if let Err(e) = process(client, &payload, tx).await {
                            println!("failed to process connection; error = {}, i={}", e,i);
                        }
                    },
                    Err(e) => {
                        println!("new connectoin error------{}----{}--------", i,e);
                        break;
                    }
                }
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

        let method = Method::from_bytes(payload.method.as_bytes()).unwrap();

        //let count = GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst);

        let request = http::request::Builder::new()
            .method(method)
            .uri(payload.path.clone())
            .header("Host", payload.host.clone())
            .header("Content-Type", "application/json")
            .body(payload.body.clone()).unwrap();

        match transport.send(request).await {
            Ok(_) => {
                if let Some(response) = transport.next().await {
                    match response {
                        Ok(response) => if response.status()==StatusCode::OK {
                            if let Err(_) = tx.send(1) {
                                break;
                            }
                        } else {
                            let _ = tx.send(1);
                            //println!("connect close......................");
                            break;
                        },

                        Err(e) => {
                            println!("{}", e);
                            break;
                        }
                    }
                }
            },

            Err(e) => {
                println!("{}", e);
                break;
            }
        }

    }

    let _ = transport.get_ref().shutdown(std::net::Shutdown::Both);

    Ok(())
}
