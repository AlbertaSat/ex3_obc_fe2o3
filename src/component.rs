use crate::{
    cmd::{self, Command, Message},
    DFGM::dfgm_handler,
};

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

pub enum Component {
    Root,
    Eps(Eps),
    Adcs(Adcs),
    Dfgm(dfgm_handler::Dfgm),
}

pub fn init() -> Vec<Component> {
    let mut components: Vec<Component> = Vec::new();

    for (index, p) in cmd::PAYLOADS.iter().enumerate() {
        assert_eq!(index, p.id, "payload index/id mismatch");
        match p.name {
            cmd::PAYLOAD_EPS => components.push(Component::Eps(Eps::configure())),
            cmd::PAYLOAD_ADCS => components.push(Component::Adcs(Adcs::configure(7))),
            cmd::PAYLOAD_DFGM => components.push(Component::Dfgm(dfgm_handler::Dfgm::configure())),
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
