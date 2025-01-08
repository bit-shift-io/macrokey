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
use crate::util;

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
    let device = match util::get_device_by_name("Usb Audio Device Mouse") {
        Ok(d) => d,
        Err(_) => return
    };
    util::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();

    loop {
        let ev = match events.next_event().await {
            Ok(e) => e,
            Err(_) => return
        };
        
        if ev.value() != 1 || ev.event_type() != EventType::KEY { continue; }
        match ev.destructure() {
            _ => {info!("mouse: {:?}", ev);}
        }
    }
}


pub async fn task_keyboard() {
    let device = match util::get_device_by_name("Usb Audio Device") {
        Ok(d) => d,
        Err(_) => return
    };
    util::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();

    loop {
        let ev = match events.next_event().await {
            Ok(e) => e,
            Err(_) => return
        };

        if ev.value() != 1 { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_F2, _) => { info!("f2 key!"); } // code 60
            _ => {info!("keyboard: {:?}", ev);}
        }
    }
}


pub async fn task_system() {
    let mut device = match util::get_device_by_name("Usb Audio Device System Control") {
        Ok(d) => d,
        Err(_) => return
    };
    util::log_device_keys(&device);
    device.grab().unwrap();// lock
    let mut events = device.into_event_stream().unwrap();

    loop {
        let ev = match events.next_event().await {
            Ok(e) => e,
            Err(_) => return
        };

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
                let _ = util::run_command("echo 'standby 0' | cec-client -s").await;
            } else {
                let _ = util::run_command("echo 'on 0' | cec-client -s").await;
            }
        }
        Err(e) => { info!("Error: {}", e); }
    }
}


pub async fn task_consumer() {
    let mut device = match util::get_device_by_name("Usb Audio Device Consumer Control") {
        Ok(d) => d,
        Err(_) => return
    };
    util::log_device_keys(&device);
    device.grab().unwrap();// lock
    // todo remap these keys to something useful
    let mut events = device.into_event_stream().unwrap();

    loop {
        let ev = match events.next_event().await {
            Ok(e) => e,
            Err(_) => return
        };

        if ev.value() != 1 { continue; }
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_CONFIG, _) => { info!("config KEY pressed!"); }
            EventSummary::Key(_, KeyCode::KEY_MAIL, _) => { info!("mail KEY pressed!"); }
            _ => {info!("consumer: {:?}", ev);}
        }
    }
}