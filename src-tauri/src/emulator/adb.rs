use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct AdbClient {
    host: String,
    port: u16,
}

impl AdbClient {
    pub fn connect_server(host: &str, port: u16) -> std::io::Result<Self> {
        Ok(Self { host: host.to_string(), port })
    }

    pub fn list_devices(&self) -> std::io::Result<Vec<String>> {
        let mut stream = TcpStream::connect(format!("{}:{}", self.host, self.port))?;
        Self::send_cmd(&mut stream, "host:devices")?;
        let mut ok = [0u8; 4];
        stream.read_exact(&mut ok)?;
        if &ok != b"OKAY" { return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to list devices")); }

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = usize::from_str_radix(std::str::from_utf8(&len_buf).unwrap_or("0"), 16).unwrap_or(0);
        
        let mut body = vec![0u8; len];
        stream.read_exact(&mut body)?;
        let output = String::from_utf8_lossy(&body);
        
        let mut serials = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 1 {
                serials.push(parts[0].to_string());
            }
        }
        Ok(serials)
    }

    fn send_cmd(stream: &mut TcpStream, cmd: &str) -> std::io::Result<()> {
        let msg = format!("{:04x}{}", cmd.len(), cmd);
        stream.write_all(msg.as_bytes())
    }

    pub fn shell(&self, serial: &str, shell_cmd: &str) -> std::io::Result<String> {
        let mut last_err = std::io::Error::new(std::io::ErrorKind::Other, "Unknown error");
        
        for _ in 0..3 {
            match TcpStream::connect(format!("{}:{}", self.host, self.port)) {
                Ok(mut stream) => {
                    let transport = format!("host:transport:{}", serial);
                    if let Err(e) = Self::send_cmd(&mut stream, &transport) { last_err = e; continue; }
                    
                    let mut ok = [0u8; 4];
                    if stream.read_exact(&mut ok).is_err() || &ok != b"OKAY" {
                        last_err = std::io::Error::new(std::io::ErrorKind::Other, format!("Không tìm thấy thiết bị {}", serial));
                        continue;
                    }

                    let cmd = format!("shell:{}", shell_cmd);
                    if let Err(e) = Self::send_cmd(&mut stream, &cmd) { last_err = e; continue; }
                    if stream.read_exact(&mut ok).is_err() || &ok != b"OKAY" {
                        last_err = std::io::Error::new(std::io::ErrorKind::Other, "Shell command failed");
                        continue;
                    }

                    let mut output = String::new();
                    let _ = stream.read_to_string(&mut output);
                    return Ok(output);
                }
                Err(e) => last_err = e,
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Err(last_err)
    }

    pub fn tap(&self, serial: &str, x: i32, y: i32) -> std::io::Result<()> {
        let _ = self.shell(serial, &format!("input tap {} {}", x, y))?;
        Ok(())
    }
}
