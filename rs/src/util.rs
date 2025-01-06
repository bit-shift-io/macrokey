pub fn check_permissions() {
    use nix::unistd::Uid;
    if !Uid::effective().is_root() {
        info!("You need to be root, or have user permissions to create uinput devices");
    }
    // TODO check uid permissions for uinput have been supplied
}


pub fn list_devices() {
    let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    // readdir returns them in reverse order from their eventN names for some reason
    devices.reverse();
    info!("Found {} devices:", devices.len());
    for (i, d) in devices.iter().enumerate() {
        info!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
    }
}


pub fn get_device_by_name(name: &str) -> Option<evdev::Device> {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    for d in devices {
        if d.name().unwrap_or("").contains(name) {
            return Some(d);
        }
    }
    None
}