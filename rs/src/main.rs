extern crate uinput_tokio;
use log::info;

use std::{thread, time::Duration};

use tokio::{self};

mod event_device;
mod util;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("== Start MacroKey ==");

    util::check_permissions();

    event_device::get_system_devices().await;

    let mut device = uinput_tokio::default()?
        .name("test")?
        .event(uinput_tokio::event::Keyboard::All)?
        .create()
        .await?;

    thread::sleep(Duration::from_secs(1));

    // device.click(&keyboard::Key::H).await?;
    // device.click(&keyboard::Key::E).await?;
    // device.click(&keyboard::Key::L).await?;
    // device.click(&keyboard::Key::L).await?;
    // device.click(&keyboard::Key::O).await?;
    // device.click(&keyboard::Key::Space).await?;
    // device.click(&keyboard::Key::W).await?;
    // device.click(&keyboard::Key::O).await?;
    // device.click(&keyboard::Key::R).await?;
    // device.click(&keyboard::Key::L).await?;
    // device.click(&keyboard::Key::D).await?;
    // device.click(&keyboard::Key::Enter).await?;

    device.synchronize().await
}