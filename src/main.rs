use std::{
    thread,  
    net::{TcpListener, TcpStream}, 
    io::{BufRead, BufReader, Write, BufWriter}
};



const ADDRESS:&str = "127.0.0.1:3030";


struct User {
    name: String
}
impl User {
    fn new(name: String) -> Self {
        User {
            name
        }
    }
}
  
fn main() {

    let listener = TcpListener::bind(ADDRESS).expect("Err to bind");

    println!("You can connect in {}", ADDRESS);

    for stream in listener.incoming() {
        let t: thread::JoinHandle<()> = thread::spawn(move || {
            println!("New thread");
            
            println!("Connection was estabileshed");
            handle_stream(stream.expect("msg"));    
        });
        // let _ = t.join();
        }
        
  

}



fn handle_stream(stream: TcpStream) {

    let mut writer: BufWriter<&TcpStream> = BufWriter::new( &stream);
    let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);

    let mut write = |msg:String| {
        let m = msg;
        let bytes = m.as_bytes();
        let _ = writer.write(bytes);
        let _ = writer.flush();
    };

    let mut read = || {
        let mut line: String = String::new();
        return match reader.read_line(&mut line) {
            Ok(_) => Ok(Some(line)),
            Err(_) => Err(())
        };
    };

    write(String::from("Welcome to the server\n"));
    write(String::from("Say your name: "));

    if let Ok(Some(s)) = read() {
        // I only must continue in case of user return his name

        let user = User::new(String::from(s.trim()));

        loop {
            write(String::from("> "));
    
            let r = read();
    
            if let Ok(Some(s)) = r {
                
                let msg = format!("<{}>: {}", user.name, s);
                println!("{}", msg);
                write(msg);
            }else {
                println!("Fails to read {:#?}", r);
            };
        
        }
    }
    
}

