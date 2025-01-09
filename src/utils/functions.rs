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
        info!("{}: {} {:?}", i, d.name().unwrap_or("<unnamed>"), get_combined_properties(d));
    }
}


/// Combines the properties and supported events of a device into a single vector.
///
/// ## Returns
///
/// A vector of strings containing the combined properties and supported
/// events of the given device.
pub fn get_combined_properties(device: &evdev::Device) -> Vec<String> {
    let mut combined_properties: Vec<_> = device.properties().into_iter().map(|x| format!("{:?}", x)).collect();
    let supported_events: Vec<_> = device.supported_events().into_iter().map(|x| format!("{:?}", x)).collect();
    combined_properties.extend(supported_events);
    combined_properties
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


/// Return a device where the given predicate is true.
///
/// ## Errors
///
/// Returns an `Err` if no device is found where the predicate is true.
/// 
/// ## Example
///
/// ```rust
/// let device_name = "device_name";
/// let properties = vec!["property1", "property2"];
/// let device = get_device_by_predicate(|d| {
///     d.name().unwrap_or("") == device_name && 
///     properties.iter().all(|prop| get_combined_properties(d).contains(&prop.to_string()))
/// })?;
/// ```
pub fn get_device_by_predicate(predicate: impl Fn(&evdev::Device) -> bool) -> Result<evdev::Device, std::io::Error> {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    for d in devices {
        if predicate(&d) {
            return Ok(d);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
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