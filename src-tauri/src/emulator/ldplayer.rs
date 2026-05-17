use std::process::Command;
use std::os::windows::process::CommandExt;
use crate::config::state::DeviceInfo;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn get_ld_path() -> String {
    "C:\\LDPlayer\\LDPlayer9\\ldconsole.exe".to_string()
}

pub fn get_ld_devices() -> std::result::Result<Vec<DeviceInfo>, String> {
    let ld_path = get_ld_path();
    let output = Command::new(&ld_path)
        .arg("list2")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|_| format!("Không thấy ldconsole.exe tại {}", ld_path))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 7 {
            let index = parts[0].parse::<i32>().unwrap_or(-1);
            let title = parts[1].to_string();
            let handle = parts[2].parse::<isize>().unwrap_or(0);
            let bind_handle = parts[3].parse::<isize>().unwrap_or(0);
            let is_in_android = parts[4] == "1";
            
            if handle != 0 && is_in_android {
                let adb_port = 5555 + (index * 2);
                devices.push(DeviceInfo {
                    index,
                    serial: format!("127.0.0.1:{}", adb_port),
                    title,
                    handle,
                    bind_handle,
                });
            }
        }
    }
    Ok(devices)
}
