use evdev::{
    EventSummary,
    EventType,
    KeyCode,
    InputEvent,
};
use tokio::{
    task::JoinSet,
    time::{
        Duration,
        sleep,
    },
};
use crate::{
    functions,
    key_event_type::KeyEventType,
    signals,
};

const TASK_ID: &str = "REMOTE";

/// Runs multiple remote control tasks as the device is split into multiple devices.
///
/// Each task is responsible for monitoring input events from the remote control and
/// performing an action based on the event.
pub async fn task() {
    info!("{}", TASK_ID);
    loop {
        let mut set = JoinSet::new();
        set.spawn(task_system());
        set.spawn(task_consumer());
        set.spawn(task_keyboard());
        set.spawn(task_mouse());
        set.join_all().await;
        
        info!("{} error, retry in 60s", TASK_ID);
        sleep(Duration::from_secs(60)).await;
    }
}


pub async fn task_mouse() {
    let mut device = try_return!(functions::get_device_by_name("Usb Audio Device Mouse"));
    functions::log_device_keys(&device);
    device.grab().unwrap();// lock
    let tx = signals::get_virtual_device_tx().await;
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        match ev.destructure() {
            // EventSummary::Key(_, KeyCode::BTN_RIGHT, _) => { // back icon 
            //     let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value());        
            //     tx.send(ie).await.unwrap();
            // }
            _ => { // passthrough
                tx.send(ev).await.unwrap();
                //info!("remote_mouse: {:?}", ev); // this give a number
                info!("mouse: {:?}", ev.destructure()); // this give keycode
            }
        }
    }
}


pub async fn task_keyboard() {
    let mut device = try_return!(functions::get_device_by_name("Usb Audio Device"));
    functions::log_device_keys(&device);
    device.grab().unwrap();// lock
    let tx = signals::get_virtual_device_tx().await;
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_F2, _) => { // windows key   
                let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value());        
                tx.send(ie).await.unwrap();
            }
            EventSummary::Key(_, KeyCode::KEY_COMPOSE, _) => { // windows key   
                let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value());        
                tx.send(ie).await.unwrap();
            }
            _ => {
                //tx.send
                info!("keyboard: {:?}", ev.destructure()); 
            }
        }
    }
}


pub async fn task_system() {
    let mut device = try_return!(functions::get_device_by_name("Usb Audio Device System Control"));
    functions::log_device_keys(&device);
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_POWER, _) => { toggle_display().await; }
            _ => {info!("system: {:?}", ev.destructure());}
        }
    }
}

async fn toggle_display() {
     match functions::run_command("echo 'pow 0' | cec-client -s -d 1").await {
        Ok(output) => { 
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("\n{}", stdout);
            if stdout.contains("power status: on") {
                let _ = functions::run_command("echo 'standby 0' | cec-client -s").await;
            } else {
                let _ = functions::run_command("echo 'on 0' | cec-client -s").await;
            }
        }
        Err(e) => { info!("Error: {}", e); }
    }
}


pub async fn task_consumer() {
    let mut device = try_return!(functions::get_device_by_name("Usb Audio Device Consumer Control"));
    functions::log_device_keys(&device);
    device.grab().unwrap();// lock
    let tx = signals::get_virtual_device_tx().await;
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_CONFIG, _) => { // alt + tab
                let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value());        
                tx.send(ie).await.unwrap();}
            EventSummary::Key(_, KeyCode::KEY_MAIL, _) => { // windows key
                let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value()); 
                tx.send(ie).await.unwrap();
            }
            // EventSummary::Key(_, KeyCode::KEY_HOMEPAGE, _) => { // home button
            //     let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_HOMEPAGE.0, ev.value()); 
            //     tx.send(ie).await.unwrap();
            // }
            _ => { // passthrough
                info!("consumer: {:?}", ev.destructure());
                tx.send(ev).await.unwrap();
            }
        }
    }
}