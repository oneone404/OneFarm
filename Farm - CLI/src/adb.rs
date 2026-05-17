use std::io::{Read, Write};
use std::net::TcpStream;

pub struct AdbClient {
    stream: TcpStream,
}

impl AdbClient {
    pub fn connect_server(host: &str, port: u16) -> std::io::Result<Self> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        Ok(Self { stream })
    }

    fn send_cmd(&mut self, cmd: &str) -> std::io::Result<()> {
        let msg = format!("{:04x}{}", cmd.len(), cmd);
        self.stream.write_all(msg.as_bytes())
    }

    pub fn tap(&mut self, serial: &str, x: i32, y: i32) -> std::io::Result<()> {
        // 1. Switch to device
        let transport = format!("host:transport:{}", serial);
        self.send_cmd(&transport)?;
        let mut ok = [0u8; 4];
        self.stream.read_exact(&mut ok)?;
        if &ok != b"OKAY" {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Transport failed"));
        }

        // 2. Execute tap command
        let tap_cmd = format!("shell:input tap {} {}", x, y);
        self.send_cmd(&tap_cmd)?;
        self.stream.read_exact(&mut ok)?;
        if &ok != b"OKAY" {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Tap shell failed"));
        }

        // Wait for execution completion
        let mut dummy = Vec::new();
        let _ = self.stream.read_to_end(&mut dummy);
        Ok(())
    }
}
