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

struct TaskState {
    modifier_enabled: bool,
    alt_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    capslock_pressed: bool,
    meta_pressed: bool,
    grave_pressed: bool,
}

impl TaskState {
    fn new() -> Self {
        TaskState {
            modifier_enabled: false,
            alt_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
            capslock_pressed: false,
            meta_pressed: false,
            grave_pressed: false
        }
    }

    fn set_alt_pressed(&mut self, pressed: bool) {
        self.alt_pressed = pressed;
        self.check_modifier();
    }

    fn set_ctrl_pressed(&mut self, pressed: bool) {
        self.ctrl_pressed = pressed;
        self.check_modifier();
    }

    fn set_shift_pressed(&mut self, pressed: bool) {
        self.shift_pressed = pressed;
        self.check_modifier();
    }

    fn set_capslock_pressed(&mut self, pressed: bool) {
        self.capslock_pressed = pressed;
        self.check_modifier();
    }

    fn set_meta_pressed(&mut self, pressed: bool) {
        self.meta_pressed = pressed;
        self.check_modifier();
    }

    fn set_grave_pressed(&mut self, pressed: bool) {
        self.grave_pressed = pressed;
    }

    fn check_modifier(&mut self) {
        if self.ctrl_pressed && self.alt_pressed { self.modifier_enabled = true; } 
        else { self.modifier_enabled = false; }
    }
}

static TASK_STATE: Lazy<Mutex<TaskState>> = Lazy::new(|| Mutex::new(TaskState::new()));


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


async fn process_input(ev: InputEvent, tx: &tokio::sync::mpsc::Sender<InputEvent>) -> () {
    let mut task_state = TASK_STATE.lock().await;

    // log
    if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { info!("{:?}", ev.destructure()); };

    // process
    match ev.destructure() {
        EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, _) => { task_state.set_alt_pressed(true); } // alt
        EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, _) => { task_state.set_ctrl_pressed(true); } // ctl
        EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, _) => {task_state.set_meta_pressed(true); } // meta
        EventSummary::Key(_, KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT, _) => { task_state.set_shift_pressed(true); } // shift
        EventSummary::Key(_, KeyCode::KEY_CAPSLOCK, _) => { task_state.set_capslock_pressed(true); } // caps lock
        EventSummary::Key(_, KeyCode::KEY_GRAVE, _) => { task_state.set_grave_pressed(true); } // ~ key
        _ => { } // no lock -> no passthrough
    }
}


async fn capture_events(device: Device) {
    //let mut device = try_return!(functions::get_device_by_name(device_name));
    functions::log_device_keys(&device);
    //device.grab().unwrap(); // lock
    let tx = signals::get_virtual_device_tx().await;
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        process_input(ev, &tx).await;
    }
}


pub async fn repeat_timer() {
    let pressed_time = 100; //ms
    let released_time = 350; //ms
    let tx = signals::get_virtual_device_tx().await;

    loop {
        let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::PRESSED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(pressed_time)).await;

        let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::RELEASED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(released_time)).await;
    }
}