/// Initialize the logger. This will use the default env_logger behavior,
/// setting the default log level to "info". The log format is also
/// customized to only include the log message, without the log level.
pub fn init_logger() {
    use std::io::Write;
    use std::env;
    use env_logger::Builder;
    env::set_var("RUST_LOG", "info");
    Builder::from_default_env()
        .format(|buf, record| {
            //let level = record.level();
            let message = record.args();
            writeln!(buf, "{}", message)
        })
        .init();
}

/// Check if the user has the necessary permissions to create uinput devices
///
/// If the user is not root, and has not added the uinput group to the user's
/// permissions, this will print a message and exit.
pub fn check_permissions() {
    use nix::unistd::Uid;
    if !Uid::effective().is_root() {
        info!("You need to be root, or have user permissions to create uinput devices");
    }
    // TODO check uid permissions for uinput have been supplied
}


/// Lists all input devices available on the system.
///
/// This function retrieves all input devices.
pub fn list_devices() {
    let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    // readdir returns them in reverse order from their eventN names for some reason
    devices.reverse();
    info!("Found {} devices:", devices.len());
    for (i, d) in devices.iter().enumerate() {
        info!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
    }
}


/// Given a device name, returns a `Device` for that device if found.
///
/// ## Errors
///
/// Returns an `Err` if no device is found with the given name.
pub fn get_device_by_name(name: &str) -> Result<evdev::Device, std::io::Error> {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    for d in devices {
        if d.name().unwrap_or("") == name {
            return Ok(d);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
}


/// Return a vector of devices where the device name matches the regex.
pub fn get_devices_by_regex(regex: &str) -> Vec<evdev::Device> {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    let regex = regex::Regex::new(regex).unwrap();
    let mut matching_devices = Vec::new();

    for d in devices {
        if regex.is_match(d.name().unwrap_or("")) {
            matching_devices.push(d);
        }
    }

    matching_devices
}


/// Logs the supported keys of a given device.
pub fn log_device_keys(device: &evdev::Device) {
    let keys = device.supported_keys().unwrap();
    info!("\nDevice: {}\nKeys: {:?}", device.name().unwrap_or(""), keys);
}


/// Run a shell command asynchronously.
pub async fn run_command(cmd: &str) -> Result<std::process::Output, std::io::Error> {
    tokio::process::Command::new("sh")
    .args(&["-c", cmd])
    .output()
    .await
}