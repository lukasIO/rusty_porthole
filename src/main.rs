#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::net::UdpSocket;
use std::env;
use std::str;
use std::thread;
use std::time::{Duration, Instant};

static MY_IP: &'static str = "10.0.179.138";

struct Config {
    shore_name: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        let shore_name = args[1].clone();

        Config{shore_name}        
    }
}

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);
    let my_shore_point = ShorePoint{name: config.shore_name, ip: None};
    let lighthouse_thread = thread::spawn(move || {run_lighthouse(&my_shore_point);});
    let shore_points = run_crows_nest();
    println!("Found shore_points: {:?}", shore_points.len());
    lighthouse_thread.join().unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShorePoint{
    name: String,
    ip: Option<std::net::SocketAddr>,
}

impl PartialEq for ShorePoint {
    fn eq(&self, other: &ShorePoint) -> bool {
        self.ip == other.ip 
        && self.name == self.name
    }
}

fn send_share_request(socket :&UdpSocket)
{
    let whos_there = "abcd";
    socket.send_to(whos_there.as_bytes(), "10.0.179.255:6666").expect("could not send broadcast");

}

fn run_crows_nest() -> Vec<ShorePoint>
{
    let socket = UdpSocket::bind(format!("{}:6667",MY_IP)).expect("could not bind to address");
    socket.set_broadcast(true).expect("set_broadcast call failed");
    let time_out_duration = Duration::new(10,0);
    let option : Option<Duration> = Option::from(time_out_duration);
    socket.set_read_timeout(option).expect("set_read_timeout call failed");
    socket.set_nonblocking(true).expect("could not set non-blocking");
    
    // thread::spawn(move || {
    //     send_share_request(&socket)
    // });
    
    let start = Instant::now();
    let mut buf = [0u8; 1024];
    let mut shore_points: Vec<ShorePoint> = Vec::new();
    while start.elapsed().as_secs() < 5 {
        send_share_request(&socket);
        let (amt, src) = match socket.recv_from(&mut buf)
        {
            Ok(r) => r,
            Err(e) => {
                println!("{}",e);
                continue;
            },
            
        };
        let filled_buffer = &buf[..amt];
        let payload = str::from_utf8(filled_buffer).expect("Error converting payload to string");

        // Convert the JSON string back to a Point.
        let mut deserialized: ShorePoint = match serde_json::from_str(&payload)
        {
            Ok(r) => r,
              Err(e) => {
                println!("{}",e);
                continue;
            }, 
        };            
        deserialized.ip = Option::from(src);                                               
       
        if !shore_points.contains(&deserialized) {
            println!("{:?}", &deserialized);
            shore_points.push(deserialized);
        }
        thread::sleep(Duration::from_millis(100));
        
    }

    return shore_points

}

fn add_to_shore_sightings(shorepoint :ShorePoint)
{

}

fn run_lighthouse(shore_point: &ShorePoint){
    // let socket = create_and_bind_socket();
    let socket = UdpSocket::bind(format!("{}:6666",MY_IP)).expect("could not bind to address");
    let mut buf = [0; 1024];
    loop {
        
        let (amt, src) = match socket.recv_from(&mut buf)
        {
            Ok(r) => r,
            Err(e) => {
                println!("{}",e);
                return ();
            },
            
        };
        let filled_buffer = &buf[..amt];
        let code_word = str::from_utf8(&filled_buffer).unwrap();
        //println!("{}: {:?}", src, code_word);
        if code_word == "abcd" {
            // println!("{}",shore_point.name);                                                   
            let serialized_shore = serde_json::to_string(shore_point).expect("Error serializing Shore Point");
            socket.send_to(serialized_shore.as_bytes(), src).expect("Error sending Shore Point");
        }
        
    }
   
}

// fn create_and_bind_socket(){
// {
    // let socket = UdpSocket::bind("127.0.0.1:6666").expect("could not bind to address!");
// }