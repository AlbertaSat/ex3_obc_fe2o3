use crate::message::{self, Command, Message};

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

pub struct Dfgm {
    port: u16,
    state: u8,
}

impl Dfgm {
    fn configure() -> Dfgm {
        Dfgm { port: 1, state: 0 }
    }

    fn dispatch(&mut self, cmd: &Command) -> Result<Message, &'static str> {
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

pub enum Component {
    Root,
    Eps(Eps),
    Adcs(Adcs),
    Dfgm(Dfgm),
    GroundStation,
}

pub fn init() -> Vec<Component> {
    let mut components: Vec<Component> = Vec::new();

    for (index, p) in message::PAYLOADS.iter().enumerate() {
        assert_eq!(index, p.id, "payload index/id mismatch");
        match p.name {
            message::PAYLOAD_EPS => components.push(Component::Eps(Eps::configure())),
            message::PAYLOAD_ADCS => components.push(Component::Adcs(Adcs::configure(7))),
            message::PAYLOAD_DFGM => components.push(Component::Dfgm(Dfgm::configure())),
            message::PAYLOAD_GNDSTN => components.push(Component::GroundStation),
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
        Component::GroundStation => Ok(cmd.status_msg(76)),
        _ => Err("Unrecognized component"),
    }
}
