use std::net::TcpListener;
use std::io::{Write,Read};
use std::net::TcpStream;

fn handle_connection(stream: &mut TcpStream){
    loop{
        let mut buffer = [0;128];
        println!("listening fo incomming PING");
        stream.read(&mut buffer).unwrap();
        stream.write_all("+PONG\r\n".as_bytes()).unwrap();
    }
    println!("Exited loop");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {

                println!("accepted new connection");
                
                handle_connection(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
