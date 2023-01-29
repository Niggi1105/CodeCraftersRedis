// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::{BufReader, BufWriter};
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    //
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                let mut writer = BufWriter::new(&_stream);
                writer.write_all("+PONG\r\n".as_bytes()).expect("Cant write back");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
