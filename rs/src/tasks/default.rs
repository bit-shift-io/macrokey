use std::{thread, time::Duration};
use uinput_tokio::event::keyboard;

use crate::event_device::EventDevice;

const TASK_ID: &str = "DEFAULT";

pub async fn task() {
    info!("{}", TASK_ID);

    // open the desired device
   // let device = devices.iter().find(|d| d.name == Some("AT Translated Set 2 keyboard".to_string())).unwrap();

    // create a new device from an existing default
    let mut device = uinput_tokio::default().unwrap()
        .name("default_task").unwrap()
        .event(uinput_tokio::event::Keyboard::All).unwrap()
        .create()
        .await.unwrap();

    thread::sleep(Duration::from_secs(1));

    device.click(&keyboard::Key::H).await.unwrap();
    device.click(&keyboard::Key::E).await.unwrap();
    device.click(&keyboard::Key::L).await.unwrap();
    device.click(&keyboard::Key::L).await.unwrap();
    device.click(&keyboard::Key::O).await.unwrap();
    device.click(&keyboard::Key::Space).await.unwrap();
    device.click(&keyboard::Key::W).await.unwrap();
    device.click(&keyboard::Key::O).await.unwrap();
    device.click(&keyboard::Key::R).await.unwrap();
    device.click(&keyboard::Key::L).await.unwrap();
    device.click(&keyboard::Key::D).await.unwrap();
    device.click(&keyboard::Key::Enter).await.unwrap();

    let _ = device.synchronize().await;

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}