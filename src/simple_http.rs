use bytes::BytesMut;
use http::{Request, Response};
//use http::header::{HeaderName, HeaderValue};
use std::{fmt, io};
use tokio_util::codec::{Decoder, Encoder};

pub struct Http;

impl Encoder for Http {
    type Item = Request<String>;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> io::Result<()> {
        use std::fmt::Write;

        write!(
            BytesWrite(dst),
            "\
            {} {} HTTP/1.1\r\n\
            Content-Length: {}\r\n\
            ",
            item.method().as_str(),
            item.uri().path(),
            item.body().len(),
        )
        .unwrap();

        for (k, v) in item.headers() {
            dst.extend_from_slice(k.as_str().as_bytes());
            dst.extend_from_slice(b": ");
            dst.extend_from_slice(v.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }

        dst.extend_from_slice(b"\r\n");
        dst.extend_from_slice(item.body().as_bytes());

        return Ok(());

        // Right now `write!` on `Vec<u8>` goes through io::Write and is not
        // super speedy, so inline a less-crufty implementation here which
        // doesn't go through io::Error.
        struct BytesWrite<'a>(&'a mut BytesMut);

        impl fmt::Write for BytesWrite<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.extend_from_slice(s.as_bytes());
                Ok(())
            }

            fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
                fmt::write(self, args)
            }
        }
    }
}

impl Decoder for Http {
    type Item = Response<()>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {

        if src.len() == 0 {
            return Ok(None);
        } else {
            //println!("decoding: {:?}", src);
        }

        let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
        let mut r = httparse::Response::new(&mut parsed_headers);
        let status = r.parse(src).map_err(|e| {
            let msg = format!("failed to parse http response: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        })?;

        let header_len = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Ok(None),
        };

        let (body_len, close) = {
            let mut len = 0;
            let mut close = false;
            for (_, header) in r.headers.iter().enumerate() {
                let k = header.name.as_bytes();
                let v = header.value;
                if k == b"content-length" || k == b"Content-Length"{
                    len = std::str::from_utf8(v).unwrap().parse::<usize>().unwrap();
                }
                if k == b"Connection" {
                    if v == b"close" {
                        //println!("connection will close *******************");
                        close = true;
                    }
                }
            }
            (len,close)
        };

        let length = header_len + body_len;
        if length > src.len() {
            return Ok(None);
        }

        let code = match r.code {
            Some(200) => 
                //StatusCode::OK
                if close {
                    "205"
                } else {
                    "200"
                },

            _ => "500",
        };

        let resp = Response::builder().status(code).body(()).unwrap();

        let _ = src.split_to(length);

        Ok(Some(resp))

    }
}
