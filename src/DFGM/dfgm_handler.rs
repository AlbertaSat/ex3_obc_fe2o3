/*
DFGM Handler receives commands related to the DFGM and performs their related action.

Communication to the DFGM is done through the DFGM interface.

TODO - What is the point of the port variable? It is set by the command, but never used.
TODO - Implement the dispatch function to handle the DFGM command to toggle 'listen_to_DFGM' flag
*/
use crate::DFGMInterface;

use crate::message::{Command, Message};
use std::sync::{Arc, Mutex};
use std::thread;

use std::fs::OpenOptions;
use std::io::prelude::*;

const DFGM_DATA_DIR_PATH: &str = "dfgm_data";

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
    /// Called in program main to create an instance of Dfgm and setup a interfacing thread,
    /// sort of like a constructor.
    pub fn configure() -> Dfgm {
        let mut dfgm = Dfgm::new();
        dfgm.setup_interface_thread();
        dfgm
    }

    /// Create thread to communicate with DFGM interface
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
        let listen_to_DFGM_clone = self.listen_to_DFGM.clone();
        thread::spawn(move || {
            loop {
                {
                    let mut interface = interface_clone.lock().unwrap();

                    let mut buffer = [0; DFGM_PACKET_SIZE];
                    match interface.receive(&mut buffer) {
                        Ok(size) => {
                            if size > 0 && listen_to_DFGM_clone {
                                //TODO - log data IF listen_to_DFGM flag is set, otherwise ignore
                                //println!("Received: {:?}", &buffer[..size]);
                                let write_result = store_dfgm_data(buffer.as_ref());
                                match write_result {
                                    Ok(_) => println!("DFGM data written to file"),
                                    Err(e) => eprintln!("Error writing data to file: {:?}", e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Receive error: {:?}", e);
                        }
                    }
                }
            }
        });
    }

    /// Right now this is how a command is received from GS, through the OBC... Later we prob want a command dispatcher for all sub systems 
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

/// Write DFGM data to a file (for now --- this may changer later if we use a db or other storage)
/// Later on we likely want to specify a path to specific storage medium (sd card 1 or 2)
/// We may also want to implement something generic to handle 'payload data' storage so we can have it duplicated, stored in multiple locations, or compressed etc.
fn store_dfgm_data(data: &[u8]) -> std::io::Result<()> {
    std::fs::create_dir_all(DFGM_DATA_DIR_PATH)?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/data", DFGM_DATA_DIR_PATH))?;
    file.write_all(data)?;
    Ok(())
}
