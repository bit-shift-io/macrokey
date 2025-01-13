use evdev::{
    Device, 
    EventType,
    EventSummary,
    InputEvent, 
    KeyCode,
    LedCode,
};
use tokio::{
    sync::Mutex,
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
use std::collections::HashMap;
use crate::{
    functions, 
    key_event_type::KeyEventType, 
    signals
};

const TASK_ID: &str = "AUTO REPEAT";
static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::new()));


#[derive(Debug)]
struct State {
    alt_pressed: bool,
    ctrl_pressed: bool,
    capslock_pressed: bool,
    meta_pressed: bool,
    repeat_events: HashMap<KeyCode, (JoinHandle<()>, InputEvent)>,
}

impl State {
    fn new() -> Self {
        State {
            alt_pressed: false,
            ctrl_pressed: false,
            capslock_pressed: false,
            meta_pressed: false,
            repeat_events: HashMap::new(),
        }
    }

    fn update_state(&mut self, ev: &InputEvent) {
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, value) => {
                self.alt_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
            }
            EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, value) => {
                self.ctrl_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
            }
            EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, value) => {
                self.meta_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
            }
            EventSummary::Led(_, LedCode::LED_CAPSL, value) => {
                self.capslock_pressed = value == 1;
            }
            _ => {}
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
        // and special keys to ignore
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, _) => { return false } // alt key
            EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, _) => { return false } // ctl key
            EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, _) => { return false } // meta
            EventSummary::Key(_, KeyCode::KEY_CAPSLOCK, _) => { return false } // caps
            _ => {}
        }

        true
    }

    fn is_active_repeat_event(&mut self, ev: InputEvent) -> bool {
        let key = KeyCode::new(ev.code());
        self.repeat_events.contains_key(&key)
    }

    fn is_stop_all_key(&mut self, ev: InputEvent) -> bool {
        let key = KeyCode::new(ev.code());
        key == KeyCode::KEY_GRAVE
    }

    fn is_toggle_pause(&mut self, ev: InputEvent) -> bool {
        // monitoring the led state, not the button itself
        if ev.event_type() == EventType::LED && ev.code() == LedCode::LED_CAPSL.0 { return true }
        false
    }

    fn is_toggle_pressed(&mut self) -> bool {
        self.capslock_pressed
    }

    fn stop_repeat_event(&mut self, key: KeyCode) {
        if let Some(value) = self.repeat_events.remove(&key) {
            value.0.abort();
        }
    }

    fn start_repeat_event(&mut self, key: KeyCode, ie: InputEvent) {
        let handle = tokio::spawn(repeat_event(ie));
        self.repeat_events.insert(key, (handle, ie));
    }

    fn stop_all_repeat_events(&mut self) {
        for value in self.repeat_events.values() {
            value.0.abort();
        }
        self.repeat_events.clear();
    }

    fn pause_all_repeat_events(&mut self) {
        for value in self.repeat_events.values() {
            value.0.abort();
        }
    }

    fn resume_all_repeat_events(&mut self) {
        let mut new_events = Vec::new();
        for value in self.repeat_events.values() {
            let handle = tokio::spawn(repeat_event(value.1.clone()));
            new_events.push((KeyCode::new(value.1.code()), (handle, value.1)));
        }
        for (key, value) in new_events {
            self.repeat_events.insert(key, value);
        }
    }
}


pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        let devices = functions::get_devices_by_regex("keyboard");

        let mut set = JoinSet::new();
        for device in devices {
            set.spawn(monitor_events(device));
        }
        set.join_all().await;
        
        info!("{} error, retry in 60s", TASK_ID);
        sleep(Duration::from_secs(60)).await;
    }
}


async fn process_input(ev: InputEvent) -> () {
    // filter unwanted events
    if ev.event_type() != EventType::KEY && ev.event_type() != EventType::LED { return };

    // log
    //info!(" > {:?}", ev.destructure());

    let mut state = STATE.lock().await;
    state.update_state(&ev);

    // timers start
    // all modifiers pressed + repeatable key + not already a repeat
    if state.all_modifiers_pressed() && state.is_repeatable(ev) && !state.is_active_repeat_event(ev) {
        state.start_repeat_event(KeyCode::new(ev.code()), ev);
    }

    // timers end
    // no modifiers pressed + repeatable key
    if !state.any_modifier_pressed() && state.is_repeatable(ev) {
        // active repeat event
        if state.is_active_repeat_event(ev) { // todo: flakey, modifiers stop working when using these... why?
            info!("{}: stop!", TASK_ID);
            state.stop_repeat_event(KeyCode::new(ev.code()));
        }

        // delete all timers
        // stop all key
        if state.is_stop_all_key(ev) {
            state.stop_all_repeat_events();
        }
    }
    // no modifiers pressed + toggle key(led)
    else if !state.any_modifier_pressed() && state.is_toggle_pause(ev) {
        match state.is_toggle_pressed() {
            true => state.pause_all_repeat_events(),
            false => state.resume_all_repeat_events(),
        }
    }
}


async fn monitor_events(device: Device) {
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
    //info!("{}: start repeat: {:?}", TASK_ID, ie.destructure());
    let key_code = ie.code();
    let press = InputEvent::new_now(EventType::KEY.0, key_code, KeyEventType::PRESSED.into());
    let release = InputEvent::new_now(EventType::KEY.0, key_code, KeyEventType::RELEASED.into());
    loop {
        tx.send(press).await.unwrap();
        sleep(Duration::from_millis(pressed_time)).await;
        tx.send(release).await.unwrap();
        sleep(Duration::from_millis(released_time)).await;
    }
}