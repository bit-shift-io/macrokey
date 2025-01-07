use evdev::{EventSummary, KeyCode};
use tokio::task::JoinSet;
use super::log;
use crate::util;

const TASK_ID: &str = "REMOTE";

pub async fn task() {
    info!("{}", TASK_ID);

    // this remote has multiple devices
    // so we split this task into each device
    // usless devices:
    // "Usb Audio Device"
    // "Usb Audio Device Mouse"
    let mut set = JoinSet::new();
    set.spawn(task_system());
    set.spawn(task_consumer());
    set.spawn(log::task("Usb Audio Device")); // debug
    set.spawn(log::task("Usb Audio Device Mouse")); // debug
    set.join_all().await;

}


pub async fn task_system() {
    // the system device is just the power button
    let mut device = util::get_device_by_name("Usb Audio Device System Control").unwrap();
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();

        // not a button press (we dont want release)
        if ev.value() != 1 { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_POWER, _) => { info!("power KEY pressed!"); }
            _ => {info!("system: {:?}", ev);}
        }
    }
}


pub async fn task_consumer() {
    // the consumer device is the keyboard
    let mut device = util::get_device_by_name("Usb Audio Device Consumer Control").unwrap();
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();

        // not a button press (we dont want release)
        if ev.value() != 1 { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_CONFIG, _) => { info!("config KEY pressed!"); }
            EventSummary::Key(_, KeyCode::KEY_MAIL, _) => { info!("mail KEY pressed!"); }
            _ => {info!("consumer: {:?}", ev);}
        }
    }
}