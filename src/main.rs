use std::{
    thread,  
    net::{TcpListener, TcpStream, SocketAddr}, 
    io::{BufRead, BufReader, Write, BufWriter}, fs::{self, File, OpenOptions}, path::Path, sync::{mpsc, Arc, Mutex}
};

use serde::{Serialize, Deserialize};



const ADDRESS:&str = "127.0.0.1:3030";
struct UserData(SocketAddr, TcpStream);

#[derive(Serialize, Deserialize)]
struct InMemoryDB {
    users: Vec<User>
}
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    addr: SocketAddr
}
impl User {
    fn new(name: String, addr: SocketAddr) -> Self {
        User {
            name,
            addr
        }
    }
}



  
fn main() {

    let listener = TcpListener::bind(ADDRESS).expect("Err to bind");

    println!("You can connect in {}", ADDRESS);
    let available_streams:Arc<Mutex<Vec<UserData>>>=Arc::new(Mutex::<Vec<UserData>>::new(vec![]));


    for stream in listener.incoming() {


        let available = available_streams.clone();

        let _: thread::JoinHandle<()> = thread::spawn(move || {
            println!("New thread");
            println!("Connection was estabileshed");
            handle_stream(stream.expect("msg"),  available);    
        });
        // let _ = t.join();
        }
        
  

}



fn handle_stream(
    stream: TcpStream,
    available_streams: Arc<Mutex<Vec<UserData>>>
) {

    {
        let mut av = available_streams.lock().unwrap();
        av.push(
            UserData(stream.peer_addr().unwrap(), stream.try_clone().unwrap())
        )

    }
    
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

    
     while let Ok(Some(s)) = read() {
        // I only must continue in case of user return his name
        
        let user = match  handle_user(String::from(s.trim()), stream.peer_addr().expect("failed to get local addr")) {
            Some(user) => user,
            _ => { 
                write(String::from("User already logged in\n"));    
                write(String::from("Type another name: "));    

                continue;
             }    
        };



        loop {

            write(String::from("> " ));
            if let Ok(Some(s)) = read() {                
                let msg = format!("<{}>: {}", user.name, s);
                send_messages(msg, available_streams.clone(), stream.peer_addr().unwrap());

              
            }
        
        }
    }
    
}

fn send_messages(
    current_message: String,
    available_streams: Arc<Mutex<Vec<UserData>>>,
    addr: SocketAddr
) {

    let streams = available_streams.lock().unwrap();
    for data in streams.iter() {

            // Itering all connected streams and send the message for all them, except by who sent

            if data.0 != addr {
            let mut writer = BufWriter::new(&data.1);
            let _ = writer.write(current_message.as_bytes()).expect("Err in write");
            let _ = writer.write("> ".as_bytes()).expect("Err in write");
            let _ = writer.flush().expect("Err in flush writer");
            
            }
            
    }
}

fn get_db() -> InMemoryDB {
    let reader = fs::read_to_string(Path::new("users.json")).expect("Failed to read file");

    let in_memory_db:InMemoryDB = if !reader.is_empty()
    { serde_json::from_str(&reader).expect("Failed to convert to json") } 
   else 
   {
       let r = r#"{"users":[]}"#;
       serde_json::from_str(r).expect("Failed to convert to json") 
   };

    in_memory_db
}

fn handle_user(name: String, addr: SocketAddr) -> Option<User> {

    let mut file = OpenOptions::new()
    .read(true)
    .create(true)
    .write(true)
    .append(false)
    .open("users.json")
    .expect("Error in open file");


    let mut in_memory_db = get_db();

    if let Some(_) = in_memory_db.users.iter().find(|x| x.name == name) {
            return None
    }

    
    let user = User::new(String::from(&name), addr);
    in_memory_db.users.push(user);

    let json = serde_json::json!(&in_memory_db);
    let value = json.to_string();
       



    let _ = file.write(value.as_bytes()).expect("Failed to write");

    Some(User::new(name, addr))

}