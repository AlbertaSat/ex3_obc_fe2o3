use crate::{
    message::{self, Command, Message},
    DFGMInterface,
    DFGM::dfgm_handler,
};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Eps {
    uart: u8,
    state: u8,
}

impl Eps {
    fn configure() -> Eps {
        Eps { uart: 7, state: 1 }
    }

    fn dispatch(&mut self, cmd: &Command) -> Result<Message, &'static str> {
        println!("Eps: opcode {0} uart {1}", cmd.opcode, self.uart);
        if cmd.oplen > 0 {
            let arg = std::str::from_utf8(&cmd.opdata).unwrap();
            println!("Eps: op data: {}", arg);
        }
        Ok(cmd.status_msg(self.state))
    }
}

pub struct Adcs {
    port: i32,
}

impl Adcs {
    fn configure(args: i32) -> Adcs {
        Adcs { port: args }
    }

    fn dispatch(&mut self, cmd: &Command) -> Result<Message, &'static str> {
        self.port = -self.port;
        println!("Adcs: opcode {0}, port {1}", cmd.opcode, self.port);

        Ok(cmd.status_msg(42))
    }
}

// const DFGM_PACKET_SIZE: usize = 1250;
// pub struct Dfgm {
//     port: u16,
//     state: u8,
//     interface: Option<Arc<Mutex<DFGMInterface>>>,
//     listen_to_DFGM: bool,
// }

// impl Dfgm {
//     fn new () -> Dfgm {
//         Dfgm {
//             port: 1,
//             state: 0,
//             interface: None, 
//             listen_to_DFGM: true,
//         }
//     }
//     fn configure() -> Dfgm {
//         let mut dfgm = Dfgm::new(); 
//         dfgm.setup_interface_thread();

//             //interface: Arc::new(Mutex::new(DFGMInterface::new_tcp("localhost:1802").unwrap())),
//         dfgm
//     }
//     //Create thread to communicate with DFGM interface

//     pub fn setup_interface_thread(&mut self) {
//         //Create new interface for TCP client connection to sim dfgm 
//         let dfgm_interface_obj = match DFGMInterface::new_tcp("localhost:1802") {
//             Ok(interface) => interface,
//             Err(e) => {
//                 eprintln!("Failed to open TCP connection for DFGM: {:?}", e);
//                 return;
//             }
//         };

//         self.interface = Some(Arc::new(Mutex::new(dfgm_interface_obj)));

//         let interface_clone = self.interface.as_ref().map(Arc::clone).unwrap();
//         thread::spawn(move || {
//             loop {
//                 {
//                     let mut interface = interface_clone.lock().unwrap();

//                     let mut buffer = [0; DFGM_PACKET_SIZE];
//                     match interface.receive(&mut buffer) {
//                         Ok(size) => {

//                             //TODO - log data IF listen_to_DFGM flag is set, otherwise ignore
//                             println!("Received: {:?}", &buffer[..size]);
//                         }
//                         Err(e) => {
//                             eprintln!("Receive error: {:?}", e);
//                         }
//                     }
//                 }

//                 // Sleep to simulate periodic work
//                 thread::sleep(Duration::from_secs(1));
//             }
//         });
//     }

//     //If the listen_to_dfgm flag is set true, then the OBC will recevie and store the data from the DFGM

//     fn dispatch(&mut self, cmd: &Command) -> Result<Message, &'static str> {
//         if cmd.oplen > 0 {
//             let arg = std::str::from_utf8(&cmd.opdata).unwrap();
//             println!("dfgm op data: {0}", arg);
//             match arg.parse::<u16>() {
//                 Ok(i) => self.port = i,
//                 Err(s) => println!("Parse failed: {}", s),
//             }
//         }
//         println!("Dfgm: opcode {}, port: {}", cmd.opcode, self.port);
//         self.state += 1;
//         Ok(cmd.status_msg(self.state))
//     }
// }

pub enum Component {
    Root,
    Eps(Eps),
    Adcs(Adcs),
    Dfgm(dfgm_handler::Dfgm),
}

pub fn init() -> Vec<Component> {
    let mut components: Vec<Component> = Vec::new();

    for (index, p) in message::PAYLOADS.iter().enumerate() {
        assert_eq!(index, p.id, "payload index/id mismatch");
        match p.name {
            message::PAYLOAD_EPS => components.push(Component::Eps(Eps::configure())),
            message::PAYLOAD_ADCS => components.push(Component::Adcs(Adcs::configure(7))),
            message::PAYLOAD_DFGM => components.push(Component::Dfgm(dfgm_handler::Dfgm::configure())),
            _ => components.push(Component::Root),
        }
    }

    components
}

pub fn dispatch_cmd(target: &mut Component, cmd: &Command) -> Result<Message, &'static str> {
    match target {
        Component::Eps(eps) => eps.dispatch(cmd),
        Component::Adcs(adcs) => adcs.dispatch(cmd),
        Component::Dfgm(dfgm) => dfgm.dispatch(cmd),
        _ => Err("Unrecognized component"),
    }
}
