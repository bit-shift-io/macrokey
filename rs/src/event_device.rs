use log::info;
use tokio::fs;
use std::io::{Error, ErrorKind};
use std::ffi::CStr;
use std::os::raw::c_char;
use nix::ioctl_read;
use nix::fcntl::{open, OFlag};
use nix::unistd::close;
use nix::sys::stat::Mode;
use std::str;

// Define the ioctl for EVIOCGNAME as a custom ioctl call
ioctl_read!(evio_get_name, b'E', 0x06, [c_char; 256]);

pub struct EventDevice {
    name: String,
    path: String,
}


impl EventDevice {
    fn new(device: &str) -> Result<Self, Error> {
        // Open the device file
        let fd = open(device, OFlag::O_RDONLY, Mode::empty())
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to open device"))?;

        // Get the device name using ioctl
        let mut nm = [0 as c_char; 256]; // Prepare the buffer for the device name
        unsafe {
            let _ = evio_get_name(fd, &mut nm);
            // if let Err(_) = evio_get_name(fd, &mut nm) {
            //     info!("Failed to get device name for: {}", device);
            // }
        }
        let name = unsafe { CStr::from_ptr(nm.as_ptr()) }
            .to_string_lossy()
            .into_owned();
    
        info!("{} -> {}", device, name);
    
        // Close the file descriptor
        close(fd).unwrap_or(());

        Ok(EventDevice {
            name,
            path: device.to_string(),
        })
    }

}

async fn is_event_device(entry: &fs::DirEntry) -> bool {
    let file_name_osstr = entry.file_name();
    let file_name = file_name_osstr.to_str().unwrap();
    if !file_name.starts_with("by-id") && !file_name.starts_with("by-path") {
        return true;
    }
    false
}


pub async fn get_system_devices() -> Vec<EventDevice> {
    let mut list = Vec::new();

    let entries = match fs::read_dir("/dev/input").await {
        Ok(entries) => entries,
        Err(_) => return list,
    };

    info!("System devices:");

    let mut entries = entries;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if is_event_device(&entry).await {
            let path = entry.path();
            let filename = path.to_string_lossy().into_owned();
            list.push(EventDevice::new(&filename).unwrap());
            //info!("{}", filename);
        }
    }

    list
}