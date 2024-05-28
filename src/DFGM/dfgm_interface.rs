
/*

Setup a TCP client to talk to the DFGM via TCP port 

*/

use std::result::Result;
use std::error::Error;
use std::net::TcpStream;
use std::io::{self, Read, Write};

const DFGM_sim_port: u16 = 1802;

trait Interface: Send + Sync {
    /// Send buffer as reference to array of bytes 
    fn send(&mut self, data: &[u8]) ->  io::Result<()>; 
    /// Place recevied bytes into buffer reference, and return number of byte recevied
    fn receive(&mut self, buffer: &mut [u8]) -> io::Result<usize>; 
}

pub struct TCPInterface {
    stream: TcpStream,
}

impl TCPInterface {
    pub fn new(address: &str) -> io::Result<Self> {
        let stream: TcpStream = TcpStream::connect(address)?;
        Ok(Self {stream})
    }
}

impl Interface for TCPInterface {
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data)
    } 

    fn receive(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buffer)
    } 
}



//DFGM interface struct can either be a TCP Interface OR a UART interface 
pub struct DFGMInterface {
    interface: Box<dyn Interface>,
}

impl DFGMInterface {
    pub fn new_tcp(address: &str) -> io::Result<Self> {
        let interface = Box::new(TCPInterface::new(address)?);
        Ok(Self {interface})
    }

    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.interface.send(data)
    }

    pub fn receive(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.interface.receive(buffer)
    }
    
}

#[cfg(tests)]
mod tests {
    use super::*;
    
    // fn start_mock_server

    #[test]
    fn test_tcp_send(){
        
    }

    #[test]
    fn test_tcp_receive(){
        
    }

}