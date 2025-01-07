use std::process::Stdio;

use evdev::{EventSummary, KeyCode};
use tokio::{process::Command, task::JoinSet};
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
    set.spawn(task_keyboard());
    set.spawn(task_mouse());
    set.join_all().await;
}


pub async fn task_mouse() {
    // the mouse device is the mouse
    let mut device = util::get_device_by_name("Usb Audio Device Mouse").unwrap();
    util::log_device_keys(&device);
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();

        // not a button press (we dont want release)
        if ev.value() != 1 { continue; }
        match ev.destructure() {
            _ => {info!("mouse: {:?}", ev);}
        }
    }
}


pub async fn task_keyboard() {
    // the keyboard device is the keyboard and numpad etc..
    let mut device = util::get_device_by_name("Usb Audio Device").unwrap();
    util::log_device_keys(&device);
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();

        // not a button press (we dont want release)
        if ev.value() != 1 { continue; }
        match ev.destructure() {
            _ => {info!("keyboard: {:?}", ev);}
        }
    }
}


pub async fn task_system() {
    // the system device is just the power button
    let mut device = util::get_device_by_name("Usb Audio Device System Control").unwrap();
    util::log_device_keys(&device);
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();

        // not a button press (we dont want release)
        if ev.value() != 1 { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_POWER, _) => { toggle_display().await; }
            _ => {info!("system: {:?}", ev);}
        }
    }
}

async fn toggle_display() {
     match util::run_command("echo 'pow 0' | cec-client -s -d 1").await {
        Ok(output) => { 
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("\n{}", stdout);

            if stdout.contains("power status: on") {
                util::run_command("echo 'standby 0' | cec-client -s").await;
            } else {
                util::run_command("echo 'on 0' | cec-client -s").await;
            }
        }
        Err(e) => { info!("Error: {}", e); }
    }
}


pub async fn task_consumer() {
    // the consumer device is the extra keys
    let mut device = util::get_device_by_name("Usb Audio Device Consumer Control").unwrap();
    util::log_device_keys(&device);
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