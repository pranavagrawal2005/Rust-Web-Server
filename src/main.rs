#![allow(unused)]
use std::{fs,
        io::{prelude::*,BufReader,Write},
        net::{TcpListener,TcpStream},
        thread,
        sync::{Arc,Mutex,mpsc},
        time::Duration};
use web_server::ThreadPool;

fn main(){
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let visitor_count = Arc::new(Mutex::new(0));
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming(){
        let stream = stream.unwrap();
        let visitor_count = Arc::clone(&visitor_count);
        pool.execute(move || {
            handle_connection(stream, visitor_count);
        });
    }
}

fn handle_connection(mut stream: TcpStream,visitor_count: Arc<Mutex<u32>>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_response,html_code,content_type) = match &request_line[..] {
        "GET / HTTP/1.1" => {
            let mut count = visitor_count.lock().unwrap();
            *count += 1;
            let contents = fs::read_to_string("server_files/web_server.html").unwrap()
                            .replace("{{visitor_count}}",&count.to_string());
            ("HTTP/1.1 200 OK",contents,"text/html")
        },
        "GET /style.css HTTP/1.1" => ("HTTP/1.1 200 OK",fs::read_to_string("server_files/style.css").unwrap(),"text/css"),
        "GET /script.js HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("server_files/script.js").unwrap()
                                    ,"application/javascript"),
        _ => ("HTTP/1.1 404 NOT FOUND",fs::read_to_string("server_files/error.html").unwrap(),"text/html")
    };

    let html_size = html_code.len();
    let response_final = format!("{status_response}\r\nContent-Length: {html_size}\r\nContent-Type: {content_type}\r\n\r\n{html_code}");

    stream.write_all(response_final.as_bytes()).unwrap();
}