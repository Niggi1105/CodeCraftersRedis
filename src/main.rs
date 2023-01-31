use std::net::TcpListener;
use std::io::{Write,Read};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::net::Shutdown;

fn handle_connection(mut stream:TcpStream){
    thread::spawn(move || {
        let mut  noreq = 0;
        loop{
            let mut buffer: [u8;128] = [0;128];
            println!("listening fo incomming PING");
            let len: usize = stream.read(&mut buffer).unwrap();
            let msg:String = String::from_utf8(buffer[..len].to_vec()).unwrap();
            println!("{:?}",msg);
            if msg == "*1\r\n$4\r\nping\r\n"{
                stream.write_all("+PONG\r\n".as_bytes()).unwrap();
                continue;
            }            
            if noreq >= 30{
                stream.shutdown(Shutdown::Both).unwrap();
            }else{
                noreq += 1;
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
