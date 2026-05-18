pub mod device;
pub mod session;
pub mod template;
pub mod script;
pub mod config;

// Re-export get_or_create_grabber to maintain external imports compatibility
pub use device::get_or_create_grabber;
