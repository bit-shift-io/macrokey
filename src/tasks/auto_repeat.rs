use evdev::{
    Device, 
    EventType,
    EventSummary,
    InputEvent, 
    KeyCode,
    LedCode,
};
use tokio::{
    sync::{
        Mutex,
        mpsc,
    },
    task::{
        JoinSet,
        JoinHandle,
    },
    time::{
        sleep,
        Duration
    }
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        Arc, 
        Condvar
    },
};
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
    capslock_pressed: bool,
    //meta_pressed: bool,
    active_events: HashMap<KeyCode, JoinHandle<()>>,
    //control_channel: (mpsc::Sender<bool>, mpsc::Receiver<bool>),
}

impl State {
    fn new() -> Self {
        State {
            alt_pressed: false,
            ctrl_pressed: false,
            capslock_pressed: false,
            //meta_pressed: false,
            active_events: HashMap::new(),
            //control_channel: mpsc::channel(1),
        }
    }

    /// Returns true if all modifier keys are currently pressed.
    /// 
    /// The modifier keys are currently defined as:
    /// 
    /// - Ctrl
    /// - Alt
    fn all_modifiers_pressed(&mut self) -> bool {
        // modifier keys here
        self.ctrl_pressed && self.alt_pressed
    }

    /// Returns true if any of the modifier keys are currently pressed.
    ///
    /// The modifier keys are currently defined as:
    ///
    /// - Ctrl
    /// - Alt
    fn any_modifier_pressed(&mut self) -> bool {
        // modifier keys here
        self.ctrl_pressed || self.alt_pressed
    }



    /// Returns true if the given event is a repeatable key press.
    ///
    /// A repeatable key press is one that is not a modifier key.
    ///
    /// The modifier keys currently checked are:
    ///
    /// - Ctrl
    /// - Alt
    fn is_repeatable(&mut self, ev: InputEvent) -> bool {
        // filter dud events
        if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { } // pass
        else { return false };

        // modifier keys here
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, _) => { return false } // alt key
            EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, _) => { return false } // ctl key
            //EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, _) => { return false} // meta
            _ => {}
        }

        true
    }

    fn is_active_event(&mut self, ev: InputEvent) -> bool {
        let key = KeyCode::new(ev.code());
        self.active_events.contains_key(&key)
    }

    fn is_stop_all_key(&mut self, ev: InputEvent) -> bool {
        let key = KeyCode::new(ev.code());
        key == KeyCode::KEY_GRAVE
    }

    fn is_toggle_key(&mut self, ev: InputEvent) -> bool {
        let key = KeyCode::new(ev.code());
        key == KeyCode::KEY_CAPSLOCK
    }

    fn is_toggle_pressed(&mut self) -> bool {
        self.capslock_pressed
    }

    fn stop_active_event(&mut self, key: KeyCode) {
        if let Some(handle) = self.active_events.remove(&key) {
            handle.abort();
        }
    }

    fn start_active_event(&mut self, key: KeyCode, handle: JoinHandle<()>) {
        self.active_events.insert(key, handle);
    }

    fn stop_all_active_events(&mut self) {
        for handle in self.active_events.values() {
            handle.abort();
        }
        self.active_events.clear();
    }

    fn pause_all_active_events(&mut self) {
        for handle in self.active_events.values() {
            handle.abort();
        }
    }

    fn resume_all_active_events(&mut self) {
        for key in self.active_events.keys() {
            //tokio::spawn(repeat_event(ev));
        }
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
    // log
    //if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { info!("{:?}", ev.destructure()); };

    let mut state = STATE.lock().await;

    // process modifiers
    set_modifier_state(&mut state, &ev);

    // timers start
    // with key modifier (ctrl + alt) + key
    if state.all_modifiers_pressed() && state.is_repeatable(ev) && !state.is_active_event(ev) {
        state.start_active_event(KeyCode::new(ev.code()), tokio::spawn(repeat_event(ev)));
    } 
    
    // timers end
    // with single key press
    // ensure no modifier keys are active
    if !state.any_modifier_pressed() && state.is_repeatable(ev) {

        // same key pressed
        if state.is_active_event(ev) {
            info!("{}: stop!", TASK_ID);
            state.stop_active_event(KeyCode::new(ev.code()));
        }

        // delete all timers
        if state.is_stop_all_key(ev) {
            info!("{}: stop all!", TASK_ID);
            state.stop_all_active_events();
        }

        // toggle timers on/off
        if state.is_toggle_key(ev) {
            if state.is_toggle_pressed() {
                info!("toggle pause");
                state.pause_all_active_events();
            } else {
                info!("resume");
                state.resume_all_active_events();
            }
        }
        
    }
}


fn set_modifier_state(state: &mut State, ev: &InputEvent) {
    match ev.destructure() {
        EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, value) => {
            state.alt_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
        }
        EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, value) => {
            state.ctrl_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
        }
        EventSummary::Led(_, LedCode::LED_CAPSL, value) => {
            state.capslock_pressed = value == 0;
        }
        _ => {}
    }
}


async fn capture_events(device: Device) {
    functions::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();
    while let Ok(ev) = events.next_event().await {
        process_input(ev).await;
    }
}


pub async fn repeat_event(ie: InputEvent) {
    let pressed_time = 100; //ms
    let released_time = 350; //ms
    let tx = signals::get_virtual_device_tx().await;

    loop {
        info!("fake press");
        // todo copy InputEvent and modify
        //let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::PRESSED.into());
        //tx.send(ie).await.unwrap();
        sleep(Duration::from_millis(pressed_time)).await;

        //let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::RELEASED.into());
        //tx.send(ie).await.unwrap();
        sleep(Duration::from_millis(released_time)).await;
    }
}