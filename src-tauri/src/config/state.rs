use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::core::capture::WgcGrabber;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub index: i32,
    pub serial: String,
    pub title: String,
    pub handle: isize,
    pub bind_handle: isize,
}

pub struct CachedTemplate {
    pub dimensions: (u32, u32),
    pub data: Arc<Vec<u8>>,
}

pub struct AppState {
    pub grabbers: Mutex<HashMap<isize, WgcGrabber>>,
    pub active_device: Mutex<Option<DeviceInfo>>,
    pub template_cache: Mutex<HashMap<String, CachedTemplate>>,
    pub cancelled_devices: Mutex<HashSet<isize>>,
    pub seed_purchase_history: Mutex<HashMap<String, u32>>,
}
