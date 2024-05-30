/*
Interface for IPC communication. Allows other processes on the OBC to communicate with a subsystem handler.

Check if fifo exists already and if not -> reader fifos create the fifo, writers will return an error

*/

use crate::cmd::Command;
use nix::unistd::mkfifo;
use std::path::Path;

use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

// For now to keep things simple they will all use hardcoded buffer size
const BUFFER_SIZE: usize = 128; //Twice the size of the 64byte command max

pub struct IPCReader {
    fifo_path_str: String,
    buffer: [u8; BUFFER_SIZE],
}

impl IPCReader {
    fn new(path_str: &str) -> IPCReader {
        IPCReader {
            fifo_path_str: path_str.to_string(),
            buffer: [0; BUFFER_SIZE],
        }
    }

    /// Check if fifo exists, if it does delete it, then create a new one
    fn setup_fifo(&self) -> Result<(), std::io::Error> {
        let path = Path::new(&self.fifo_path_str);
        if path.exists() {
            std::fs::remove_file(&self.fifo_path_str)?;
        }
        match mkfifo(self.fifo_path_str.as_str(), nix::sys::stat::Mode::S_IRWXU) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    //Get a message from the fifo. Too keep things simple just return the byte array
    async fn read(&self) -> Result<(), std::io::Error> {
        // Open the named pipe for reading
        let mut fifo = OpenOptions::new()
            .read(true)
            .open(self.fifo_path_str.as_str())
            .await?;
        let mut buffer = vec![0; 1024];

        loop {
            let n = fifo.read(&mut buffer).await?;
            if n == 0 {
                continue;
            }
            println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
        }
    }
}

pub struct IPCWriter {
    fifo_path_str: String,
}

impl IPCWriter {
    fn new(path_str: &str) -> IPCWriter {
        IPCWriter {
            fifo_path_str: path_str.to_string(),
        }
    }

    async fn write(&self, message: &[u8]) -> Result<(), std::io::Error> {
        let mut fifo = OpenOptions::new()
            .write(true)
            .open(self.fifo_path_str.as_str())
            .await?;
        fifo.write_all(message).await?;
        println!("Message written to the pipe");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task;

    #[tokio::test]
    async fn test_ipc_reader_writer() -> Result<(), Box<dyn std::error::Error>> {
        let fifo_path = "/tmp/rust_ipc_fifo";

        let reader = IPCReader::new(fifo_path);
        reader.setup_fifo()?;

        let writer = IPCWriter::new(fifo_path);

        let reader_task = task::spawn(async move {
            reader.read().await.unwrap();
        });

        let writer_task = task::spawn(async move {
            let message = b"Hello from the writer process!";
            writer.write(message).await.unwrap();
        });

        reader_task.await?;
        writer_task.await?;

        Ok(())
    }
}