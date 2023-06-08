use std::error;
use std::fmt;

use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use words;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 7878))?;

    for stream in listener.incoming() {
        let stream = stream?;
        handle(stream)?;
    }

    Ok(())
}

fn handle(mut stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&mut stream);
    let lines: Vec<_> = reader
        .lines()
        .take_while(|line| line.as_ref().map_or(true, |line| !line.is_empty()))
        .collect::<std::result::Result<_, _>>()?;

    println!("{lines:#?}");

    let Some(request_line) = lines
        .iter()
        .find(|line| 
            line.starts_with("GET") || line.starts_with("HEAD") || line.starts_with("OPTIONS")
        ) else {
        return Err("no request line".into());
    };

    let request = parse_request_line(&request_line)?;
    match request.method {
        Method::Options => {
            handle_cors(&mut stream)?;
            return Ok(());
        }
        _ => {}
    }
    match &*request.resource {
        "/word" => handle_word(&mut stream)?,
        _ => handle_not_found(&mut stream)?,
    }
    Ok(())
}

fn handle_cors(stream: &mut TcpStream) -> Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    )?;

    Ok(())
}

fn handle_not_found(stream: &mut TcpStream) -> Result<()> {
    write!(stream, "HTTP/1.1 404 Not Found\r\n")?;

    Ok(())
}

fn handle_word(stream: &mut TcpStream) -> Result<()> {
    let word = words::pick_random_word();
    let word = words::to_str(&word);
    let json = format!("{{ \"value\": \"{word}\" }}");

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{json}",
        length = json.len()
    )?;

    Ok(())
}

fn parse_request_line(req: &str) -> Result<Request> {
    let mut req = req.split_ascii_whitespace();
    let method = req.next().unwrap();
    let resource = req.next().unwrap();

    Ok(Request {
        method: match method {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            _ => return Err("unknown method".into()),
        },
        resource: resource.to_string(),
    })
}

#[derive(Debug)]
struct Request {
    method: Method,
    resource: String,
}

#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Options,
    Head,
}

struct Error(String);

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
