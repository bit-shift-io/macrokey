use tokio::fs;
use std::io::{Error, ErrorKind};
use std::ffi::CStr;
use std::path::Path;
use std::os::raw::c_char;
use nix::ioctl_read;
use nix::fcntl::{open, OFlag};
use nix::unistd::close;
use nix::fcntl::readlink;
use nix::sys::stat::Mode;
use std::str;
use std::fmt;
use nix::ioctl_write_int;
use nix::errno::Errno;
use evdev::{Device, Key};

const INPUT_PATH: &str = "/dev/input/";
const BY_ID_PATH: &str = "/dev/input/by-id/";

ioctl_read!(evio_get_name, b'E', 0x06, [c_char; 256]);
ioctl_read!(evio_get_phys, b'E', 0x07, [c_char; 256]);
ioctl_write_int!(eviocgrab, b'E', 0x90);


#[derive(Debug, PartialEq)]
pub struct EventDevice {
    pub name: Option<String>, // may have no name
    pub path: String,
    pub lock_type: LockType,
    //pub lock: bool,
    pub file_device: i32,
    //id: Option<String>, // may not have id
    //manufacturer: Option<String>, // may not have manufacturer
}

impl EventDevice {
    async fn new(device_path: &str) -> Result<Self, Error> {
        // Open the device file
        let fd = open(device_path, OFlag::O_RDONLY, Mode::empty())
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to open device"))?;

        // Get the device name using ioctl
        let mut nm = [0 as c_char; 256]; // Prepare the buffer for the device name
        let name = unsafe {
            if evio_get_name(fd, &mut nm).is_ok() {
                Some(CStr::from_ptr(nm.as_ptr()).to_string_lossy().into_owned())
            } else {
                None
            }
        };

        // Get the manufacturer using ioctl
        // let mut phys = [0 as c_char; 256]; // Prepare the buffer for the manufacturer
        // let manufacturer = unsafe {
        //     if evio_get_phys(fd, &mut phys).is_ok() {
        //         Some(CStr::from_ptr(phys.as_ptr()).to_string_lossy().into_owned())
        //     } else {
        //         None
        //     }
        // };
    
        // Close the file descriptor
        close(fd).unwrap_or(());

        // Get device IDs from /dev/input/by-id/
        //let id = get_device_id(device_path).await?;

        // Extract the file name from the device path
        let path = Path::new(device_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

            let event_device = EventDevice {
                name,
                path,
                lock_type: LockType::None,
                file_device: fd,
                //id,
            };
    
            info!("{}", event_device);
    
            Ok(event_device)
    }


    pub fn open(&mut self, lock_type: LockType) {
        info!("Opening device: {}", self.path);
        self.lock_type = lock_type.clone();

        // Open the device file
        let fd = open(self.get_path().as_str(), OFlag::O_RDONLY, Mode::empty())
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to open device"))
            .unwrap();

        self.file_device = fd;

        // lock/grab all input
        if lock_type == LockType::Lock || lock_type == LockType::Map {
            let result = unsafe { eviocgrab(fd, true as u64) };
            match result {
                Ok(_) => info!("Exclusive access: SUCCESS"),
                Err(Errno::EBUSY) => info!("Exclusive access: FAILURE - Device is busy"),
                Err(e) => info!("Exclusive access: FAILURE - {:?}", e),
            }
        }
    }

    pub fn close(&mut self) {
        info!("Closing device: {}", self.path);
        close(self.file_device).unwrap_or(());
    }

    pub fn is_open(&self) -> bool {
        self.file_device != -1
    }

    pub fn get_path(&self) -> String {
        format!("{}{}", INPUT_PATH, self.path)
    }

}


impl fmt::Display for EventDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.path,
            self.name.clone().unwrap_or_else(|| "<No Name>".to_string()),
            //self.id.clone().unwrap_or_else(|| "No ID".to_string()),
            //self.manufacturer.clone().unwrap_or_else(|| "No Manufacturer".to_string())
        )
    }
}


async fn get_device_id(device_path: &str) -> Result<Option<String>, Error> {
    // Iterate over all entries in the directory
    let mut entries = fs::read_dir(BY_ID_PATH).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();

        // Check if the entry is a symbolic link
        // these should all be symbolic links!
        if entry_path.is_symlink() { // irrelevant?

            match readlink(&entry_path) {
                Ok(target) => {
                    // Join the relative path with the directory of the original entry_path
                    let full_target_path = entry_path.parent().unwrap().join(target);
                   
                    // Canonicalize the full target path to resolve any symbolic links and remove redundant components
                    let canonical_target_path = fs::canonicalize(full_target_path).await?;

                    //info!("{} | {:?} | {}", entry_path.display(), canonical_target_path, device_path);
                    
                     // Check if the symbolic link target matches the device path
                     if canonical_target_path.to_str() == Some(device_path) {
                        //return Ok(canonical_target_path.to_str().map(|s| s.to_string()));
                        if let Some(file_name) = entry.file_name().to_str() {
                            return Ok(Some(file_name.to_string()));
                        }
                    }
                }
                Err(e) => info!("Failed to read link {:?}: {:?}", entry_path, e),
            }

        }
    }

    Ok(None)
}


async fn is_event_device(entry: &fs::DirEntry) -> bool {
    let file_name_osstr = entry.file_name();
    let file_name = file_name_osstr.to_str().unwrap();
    if !file_name.starts_with("by-id") && !file_name.starts_with("by-path") {
        return true;
    }
    false
}


pub async fn get_input_devices() -> Vec<EventDevice> {
    let mut list = Vec::new();

    let entries = match fs::read_dir(INPUT_PATH).await {
        Ok(entries) => entries,
        Err(_) => return list,
    };

    info!("System devices:");

    let mut entries = entries;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if is_event_device(&entry).await {
            let path = entry.path();
            let filename = path.to_string_lossy().into_owned();
            list.push(EventDevice::new(&filename).await.unwrap());
            //info!("{}", filename);
        }
    }

    list
}


#[derive(Debug, PartialEq, Clone)]
pub enum LockType {
    None, // dont lock device
    Lock, // full lock of device
    Map, // lock specific mapped keys
}