use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufReader, BufRead};
use crossbeam_channel::{Sender, Receiver};

use crate::file_cache::FileCache;


pub fn accept_to_channel(channel_out: Sender<TcpStream>) {

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => channel_out.send(stream).unwrap(),
            Err(e) => println!("Could not accept incoming connection: {}", e)
        }
    }
}

pub fn write_response(stream: &mut TcpStream, status_code: &str, mime_type: &str, payload: &[u8])  {
    let header = format!("HTTP/1.1 {}\nContent-Length: {}\nContent-Type: {}\n\n", status_code, mime_type, payload.len());
    stream.write(header.as_bytes()).unwrap();
    stream.write(payload).unwrap();
}

pub fn process_from_channel(channel_in: Receiver<TcpStream>, file_cache: &FileCache) {

    // buffer used for receiving from channel
    
    for stream in channel_in {

        let mut buf_reader = BufReader::new(stream);
        let mut buf = String::new();

        let mut get_param = String::from("/index.html");

        loop { // iterates over all lines
            buf.clear();
            let line = match buf_reader.read_line(&mut buf) {
                Ok(0) => break, // stream reached EOF
                Ok(_) => {
                    // read successful
                    buf.pop(); // remove '\n' from end of buffer
                    buf.pop(); // remove '\r' from end of buffer
                    buf.clone()
                },
                Err(_) => break // stream broke
            };

            // println!("got line: \"{}\"", line);

            let params: Vec<&str> = line.split_ascii_whitespace().collect();
            if params.len() >= 2 && params[0] == "GET" {
                get_param = String::from(params[1]);
            }

            if line.is_empty() {
                println!("requested file: {}", get_param);

                // try to find file
                match file_cache.get_file(&get_param) {
                    Some(file) =>   write_response(buf_reader.get_mut(), "200 OK", &file.mime_type, &file.payload),
                    None =>         write_response(buf_reader.get_mut(), "404 Not Found", "text/html", "Not Found".as_bytes())
                };

                break;
            }
        }
    }
}