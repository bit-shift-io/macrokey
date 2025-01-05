use log::info;
use nix::unistd::Uid;

pub fn check_permissions() {
    if !Uid::effective().is_root() {
        info!("You need to be root, or have user permissions to create uinput devices");
    }
}