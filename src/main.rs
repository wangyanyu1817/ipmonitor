use encoding::{all::{GBK}, DecoderTrap, Encoding,};
use log::{info,error};
use log4rs;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use std::vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Address {
    ip: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Monitor {
    project: String,
    ipaddress: Vec<Address>,
}

// fn read_json_typed(raw_json: &str) -> Monitor {
//     let parsed: Monitor = serde_json::from_str(raw_json).unwrap();
//     return parsed
// }
fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Monitor, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}
fn main() {
    log4rs::init_file("monitor.yml", Default::default()).unwrap();
    let parsed = read_user_from_file("monitor.json").unwrap();
    info!(
        "\n\n The name of the project is: {}",
        parsed.project
    );
    let th: Vec<JoinHandle<_>> = parsed.ipaddress.into_iter().map(|addr| {
        let child = Command::new("ping")
            .arg(addr.ip)            
            .arg("-t")
            .stdout(Stdio::piped())
            .spawn()
            .expect("start ping failed");
        let mut out = BufReader::new(child.stdout.unwrap());
        std::thread::spawn(move || {
            let mut buf = vec![];
            loop {
                match out.read_until('\n' as u8, &mut buf) {
                    Ok(_i) => {
                        info!("{} {}", addr.name, GBK.decode(&buf, DecoderTrap::Strict).unwrap().replace("\r\n", ""));
                    },
                    Err(i) => error!("{}", i),
                }
                buf.clear();
                sleep(Duration::from_secs(1));
            } 
        })
    }).collect();
    th.into_iter().for_each(|t|{
        t.join().unwrap();
    });
}
