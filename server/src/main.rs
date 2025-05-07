use std::{
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader},
};

const END_MESSAGE: &str = "\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                handle_stream(&stream); 
                println!("Successfully handled stream");
            },
            Err(e) => eprintln!("Error in request: {}", e)
        }
    }
}

fn handle_stream(stream: &TcpStream) {
    // print the request
    let body = recieve_request(&stream);

    println!("Recieved this body: {}", body);

    // send http success
    send_success(&stream);
}

fn send_success(
    mut stream: &TcpStream,
) {
    // turn the response to a string
    let mut response_string = String::from("connection success");
    response_string.push_str(END_MESSAGE);

    // send the response
    stream.write_all(response_string.as_bytes()).unwrap();
}

fn recieve_request(stream: &TcpStream) -> String {
    // create the buffer reader
    let mut buf_reader = BufReader::new(stream);

    // create the string the body goes into
    let mut body = String::new();

    // read the contents into the string
    while let Ok(bytes_read) = buf_reader.read_line(&mut body) {
        // once read end character then break
        if bytes_read == 0 || body.ends_with(END_MESSAGE) {
            break;
        }
    }
    body
}
