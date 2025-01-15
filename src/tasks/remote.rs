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

#[derive(Clone, Debug)]
enum ID {
    Keyboard,
    Mouse,
    Consumer,
    System,
}

/// Runs multiple remote control tasks as the device is split into multiple devices.
///
/// Each task is responsible for monitoring input events from the remote control and
/// performing an action based on the event.
pub async fn task() {
    info!("{}", TASK_ID);
    loop {
        let mut set = JoinSet::new();
        set.spawn(capture_events("Usb Audio Device System Control", ID::System));
        set.spawn(capture_events("Usb Audio Device Consumer Control", ID::Consumer));
        set.spawn(capture_events("Usb Audio Device", ID::Keyboard));
        set.spawn(capture_events("Usb Audio Device Mouse", ID::Mouse));
        set.join_all().await;
        
        info!("{} error, retry in 60s", TASK_ID);
        sleep(Duration::from_secs(60)).await;
    }
}


async fn process_input(id: ID, ev: InputEvent, tx: &tokio::sync::mpsc::Sender<InputEvent>) -> () {
    // log + filter pressed events
    if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { info!("{:?}: {:?}", id, ev.destructure()); };

    // process
    match ev.destructure() {
        EventSummary::Key(_, KeyCode::KEY_POWER, value) => { // power button
            if value == KeyEventType::PRESSED { toggle_cec_display().await; }
        } 

        EventSummary::Key(_, KeyCode::KEY_F2, _) => { // windows icon  -> windows key    
            tx.send(InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value())).await.unwrap();
        }
        // EventSummary::Key(_, KeyCode::KEY_HOMEPAGE, _) => { // home icon     
        //     tx.send(InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, ev.value())).await.unwrap();
        // }
        EventSummary::Key(_, KeyCode::KEY_COMPOSE, _) => { // menu icon -> left mouse      
            tx.send(InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.0, ev.value())).await.unwrap();
        }
        EventSummary::Key(_, KeyCode::KEY_CONFIG, _) => { // media icon -> browser     
            tx.send(InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_WWW.0, ev.value())).await.unwrap();
        }
        EventSummary::Key(_, KeyCode::KEY_MAIL, _) => { // exclamation mark icon -> atl + tab
            tx.send(InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_SEARCH.0, ev.value())).await.unwrap();
        }
        _ => { tx.send(ev).await.unwrap(); } // passthrough
    }
}


async fn capture_events(device_name:&str, id: ID) {
    let mut device = try_return!(functions::get_device_by_name(device_name));
    functions::log_device_keys(&device);
    device.grab().unwrap_or_default(); // lock - todo: can crash here if device locked
    let tx = signals::get_virtual_device_tx().await;
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        process_input(id.clone(), ev, &tx).await;
    }
}


async fn toggle_cec_display() {
    info!("{}: toggle cec", TASK_ID);
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