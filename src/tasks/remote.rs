use evdev::{
        EventSummary,
        EventType,
        KeyCode,
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
    };

const TASK_ID: &str = "REMOTE";

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
    let device = try_return!(functions::get_device_by_name("Usb Audio Device"));
    functions::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED || ev.event_type() != EventType::KEY { continue; }
        match ev.destructure() {
            _ => {info!("mouse: {:?}", ev);}
        }
    }
}


pub async fn task_keyboard() {
    let device = try_return!(functions::get_device_by_name("Usb Audio Device"));
    functions::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_F2, _) => { info!("f2 key!"); } // code 60
            _ => { info!("keyboard: {:?}", ev); }
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
            _ => {info!("system: {:?}", ev);}
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
    // todo remap these keys to something useful
    let mut events = device.into_event_stream().unwrap();

    while let Ok(ev) = events.next_event().await {
        if ev.value() != KeyEventType::PRESSED { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_CONFIG, _) => { info!("config KEY pressed!"); }
            EventSummary::Key(_, KeyCode::KEY_MAIL, _) => { info!("mail KEY pressed!"); }
            _ => {info!("consumer: {:?}", ev);}
        }
    }
}