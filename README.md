### yaya - a HTTP benchmarking tool
yaya is a modern HTTP benchmarking tool capable of generating significant load when run on a single multi-core CPU. It write in rust.
#### basic usage
> ./yaya -c100 -d10 -XPOST -b'{"hello":"world"}' http://localhost:8080

This runs a benchmark for 10 seconds, keeping 100 HTTP connections open, posting a simple json.
#### command line options

```
USAGE:
    yaya [OPTIONS] <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --body <body>                  http body
    -c, --connections <connections>    Connections to keep open
    -d, --duration <duration>          Duration(seconds) of test
    -X, --method <method>              http method: GET,POST,DELETE,PUT...

ARGS:
    <url>    url to test
```
#### How to build
1. install rust: https://rustup.rs/
2. cd yaya && cargo build --release
