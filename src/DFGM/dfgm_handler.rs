/*Handler receives commands and performs the related action

When talking to the DFGM it uses the DFGM interface
*/
use crate::DFGMInterface;

use crate::message::{Command, Message};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DFGM_PACKET_SIZE: usize = 1250;
pub struct Dfgm {
    port: u16,
    state: u8,
    interface: Option<Arc<Mutex<DFGMInterface>>>,
    listen_to_DFGM: bool,
}

impl Dfgm {
    pub fn new() -> Dfgm {
        Dfgm {
            port: 1,
            state: 0,
            interface: None,
            listen_to_DFGM: true,
        }
    }
    pub fn configure() -> Dfgm {
        let mut dfgm = Dfgm::new();
        dfgm.setup_interface_thread();

        //interface: Arc::new(Mutex::new(DFGMInterface::new_tcp("localhost:1802").unwrap())),
        dfgm
    }
    //Create thread to communicate with DFGM interface

    pub fn setup_interface_thread(&mut self) {
        //Create new interface for TCP client connection to sim dfgm
        let dfgm_interface_obj = match DFGMInterface::new_tcp("localhost:1802") {
            Ok(interface) => interface,
            Err(e) => {
                eprintln!("Failed to open TCP connection for DFGM: {:?}", e);
                return;
            }
        };

        self.interface = Some(Arc::new(Mutex::new(dfgm_interface_obj)));

        let interface_clone = self.interface.as_ref().map(Arc::clone).unwrap();
        thread::spawn(move || {
            loop {
                {
                    let mut interface = interface_clone.lock().unwrap();

                    let mut buffer = [0; DFGM_PACKET_SIZE];
                    match interface.receive(&mut buffer) {
                        Ok(size) => {
                            if size > 0 {
                                //TODO - log data IF listen_to_DFGM flag is set, otherwise ignore
                                println!("Received: {:?}", &buffer[..size]);
                            }
                        }
                        Err(e) => {
                            eprintln!("Receive error: {:?}", e);
                        }
                    }
                }

                // Sleep to simulate periodic work
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn dispatch(&mut self, cmd: &Command) -> Result<Message, &'static str> {
        if cmd.oplen > 0 {
            let arg = std::str::from_utf8(&cmd.opdata).unwrap();
            println!("dfgm op data: {0}", arg);
            match arg.parse::<u16>() {
                Ok(i) => self.port = i,
                Err(s) => println!("Parse failed: {}", s),
            }
        }
        println!("Dfgm: opcode {}, port: {}", cmd.opcode, self.port);
        self.state += 1;
        Ok(cmd.status_msg(self.state))
    }
}
