/*
Interface for IPC communication. Allows other processes on the OBC to communicate with a subsystem handler.

Creation of a IPCReader checks if fifo exists already and if not creates one.

Anything that uses an IPC reader must also have the FifoDataHandler trait implemented, which contains use
case specific logic on how the data read from the fifo is handled.


TODO - build fifo paths always in /tmp/fifos/ , then append the fifo name provided, that way all FSW fifos are
in their own directory
*/

use nix::unistd::mkfifo;
use std::fs::{remove_file, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;
use std::sync::Arc;

const MAX_FIFO_BUFFER_SIZE: usize = 1024;
const FIFO_DIR_PREPEND: &str = "/tmp/fifo-";

/// Contains a callback fired when data is read from the fifo
/// Handlers are to be defined by whatever is using the IPCReader (processes will do different things with the data)
trait FifoDataHandler: Send + Sync + 'static {
    fn handle_fifo_input(&self, data: &[u8]);
}

pub struct IpcReader {
    fifo_path_str: String,
    data_handler: Arc<dyn FifoDataHandler>,
}

impl IpcReader {
    fn new(fifo_name: &str, handler: Arc<dyn FifoDataHandler>) -> Result<IpcReader, Error> {
        let fifo_path = format!("{}{}", FIFO_DIR_PREPEND, fifo_name); 
        let reader = IpcReader {
            fifo_path_str: fifo_path.to_string(),
            data_handler: handler,
        };
        reader.setup_fifo()?;
        Ok(reader)
    }

    /// Check if fifo exists, if it does delete it, then create a new one
    fn setup_fifo(&self) -> Result<(), Error> {
        let path = Path::new(&self.fifo_path_str);
        if path.exists() {
            remove_file(&self.fifo_path_str)?;
        }
        match mkfifo(self.fifo_path_str.as_str(), nix::sys::stat::Mode::S_IRWXU) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        // Open the named pipe for reading
        let path_obj = Path::new(self.fifo_path_str.as_str());
        let mut fifo = OpenOptions::new().read(true).open(path_obj)?;

        let n = fifo.read(buffer)?;
        return Ok(n);
    }

    /// Begin a thread which calls the Handlers 'handle_fifo_input' fxn when data is read 
    fn start_reader_thread(&self) {
        let fifo_path_str_clone = self.fifo_path_str.clone();
        let handler = Arc::clone(&self.data_handler);
        std::thread::spawn(move || {
            let mut buffer = vec![0; MAX_FIFO_BUFFER_SIZE];
            let path = Path::new(&fifo_path_str_clone);
            let mut fifo = OpenOptions::new()
                .read(true)
                .open(path)
                .expect("Failed to open FIFO for reading");
            loop {
                match fifo.read(&mut buffer) {
                    Ok(n) => {
                        if n > 0 {
                            handler.handle_fifo_input(&buffer[..n]);
                        }
                    }
                    Err(e) => {
                        println!("Error reading FIFO: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

pub struct IpcWriter {
    fifo_path_str: String,
}

impl IpcWriter {
    fn new(fifo_name: &str) -> IpcWriter {
        let fifo_path = format!("{}{}", FIFO_DIR_PREPEND, fifo_name); 
        IpcWriter {
            fifo_path_str: fifo_path.to_string(),
        }
    }

    fn write(&self, data: &[u8]) -> Result<(), std::io::Error> {
        let path_obj = Path::new(self.fifo_path_str.as_str());
        let mut fifo = OpenOptions::new().write(true).open(path_obj)?;
        fifo.write_all(data)?;
        println!("Message written to the pipe");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    struct TestHandler {
        received_data: Arc<Mutex<Vec<u8>>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                received_data: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl FifoDataHandler for TestHandler {
        fn handle_fifo_input(&self, data: &[u8]) {
            let mut received_data = self.received_data.lock().unwrap();
            received_data.extend_from_slice(data);
            println!("Fifo handler callback received data: {:?}", received_data);
        }
    }

    #[tokio::test]
    async fn test_ipc_reader_writer() -> Result<(), Box<dyn std::error::Error>> {
        let fifo_name = "rust_ipc_test";

        let handler = Arc::new(TestHandler::new());
        let reader = IpcReader::new(fifo_name, handler.clone()).unwrap();

        reader.start_reader_thread();

        // Ensure reader thread is ready
        thread::sleep(Duration::from_secs(1));

        let writer = IpcWriter::new(fifo_name);

        let test_data = b"This is from the writer";
        writer.write(test_data)?;

        // Give some time for the reader to process the message
        thread::sleep(Duration::from_secs(1));

        let received_data = handler.received_data.lock().unwrap().clone();
        assert_eq!(received_data, test_data);

        Ok(())
    }
}
