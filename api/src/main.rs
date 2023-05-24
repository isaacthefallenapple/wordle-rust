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

    let Some(request_line) = reader
        .lines()
        .find(|line| line.as_ref().map_or(false, |line| line.starts_with("GET"))) else {
        return Err("no request line".into());
    };

    let request_line = request_line?;
    let path = request_line.split_ascii_whitespace().nth(1).unwrap();
    println!("{path}");
    match path {
        "/word" => handle_word(&mut stream)?,
        _ => handle_not_found(&mut stream)?,
    }
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
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{json}",
        length = json.len()
    )?;

    Ok(())
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
