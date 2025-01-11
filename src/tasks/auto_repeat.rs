use evdev::{
    Device, 
    EventType,
    EventSummary,
    InputEvent, 
    KeyCode
};
use tokio::{
    sync::Mutex,
    task::JoinSet,
    time::{
        sleep,
        Duration
    }
};
use once_cell::sync::Lazy;
use crate::{
    functions, 
    key_event_type::KeyEventType, 
    signals
};

const TASK_ID: &str = "AUTO REPEAT";

#[derive(Debug)]
struct State {
    alt_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    capslock_pressed: bool,
    meta_pressed: bool,
    grave_pressed: bool,
}

impl State {
    fn new() -> Self {
        State {
            alt_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
            capslock_pressed: false,
            meta_pressed: false,
            grave_pressed: false
        }
    }

    fn is_modifier_pressed(&mut self) -> bool {
        self.ctrl_pressed && self.alt_pressed
    }
}

static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::new()));


pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        let devices = functions::get_devices_by_regex("keyboard");
        if devices.len() == 0 {
            info!("{} No devices found matching: {}", TASK_ID, "keyboard");
            return;
        }

        let mut set = JoinSet::new();
        for device in devices {
            set.spawn(capture_events(device));
        }
        set.join_all().await;
        
        info!("{} error, retry in 60s", TASK_ID);
        sleep(Duration::from_secs(60)).await;
    }
}


async fn process_input(ev: InputEvent) -> () {
    let mut state = STATE.lock().await;

    // log
    if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { info!("{:?}", ev.destructure()); };

    // process
    match ev.destructure() {
        EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, value) => { state.alt_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // alt
        EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, value) => { state.ctrl_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // ctl
        EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, value) => {state.meta_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // meta
        EventSummary::Key(_, KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT, value) => { state.shift_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // shift
        EventSummary::Key(_, KeyCode::KEY_CAPSLOCK, value) => { state.capslock_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // caps lock
        EventSummary::Key(_, KeyCode::KEY_GRAVE, value) => { state.grave_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT; } // ~ key
        _ => { return } // no lock -> no passthrough
    }

    // spawn task
    if state.is_modifier_pressed() {
        info!("{}: modifier!!", TASK_ID);
        //let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::PRESSED.into());
        //tokio::spawn(repeat_timer(ie));
    }
}


async fn capture_events(device: Device) {
    functions::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        process_input(ev).await;
    }
}


pub async fn repeat_timer(ie: InputEvent) {
    let pressed_time = 100; //ms
    let released_time = 350; //ms
    let tx = signals::get_virtual_device_tx().await;

    loop {
        // todo copy InputEvent and modify
        //let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::PRESSED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(pressed_time)).await;

        //let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::RELEASED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(released_time)).await;
    }
}