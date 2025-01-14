use evdev::{
    Device, 
    EventType,
    EventSummary,
    InputEvent, 
    KeyCode,
    LedCode,
};
use tokio::{
    time::{
        sleep,
        Duration
    }
};
use crate::{
    functions, 
    key_event_type::KeyEventType, 
};

const TASK_ID: &str = "HOTKEYS";

/// Bind apps to keys
pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        // predicate example
        // get by name, and not a pointer(mouse)
        // let device = try_return!(functions::get_device_by_predicate(|d| {
        //     d.name().unwrap_or("") == "keyboard" && 
        //     !functions::get_combined_properties(d).contains(&"POINTER".to_string())
        // }));

        let device = functions::get_device_by_name("AT Translated Set 2 keyboard").unwrap();
        monitor_events(device).await;

        info!("{} error, retry in 60s", TASK_ID);
        sleep(Duration::from_secs(60)).await;
    }
}


async fn monitor_events(device: Device) {
    functions::log_device_keys(&device);
    let mut events = device.into_event_stream().unwrap();
    // each device can have its own state
    let mut state = State::new();
    while let Ok(ev) = events.next_event().await {
        // filter unwanted events, reduce locks
        if ev.event_type() != EventType::KEY && ev.event_type() != EventType::LED { continue };
        state.process_input(ev).await;
    }
}


#[derive(Debug)]
struct State {
    alt_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    capslock_pressed: bool,
    meta_pressed: bool,
}

impl State {
    fn new() -> Self {
        State {
            alt_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
            capslock_pressed: false,
            meta_pressed: false,
        }
    }

    async fn process_input(&mut self, ev: InputEvent) -> () {
        // log
        //info!(" > {:?}", ev.destructure());
        self.update_state(&ev);

        // set hotkey actions here
        // all modifiers pressed + hotkey
        if self.all_modifiers_pressed() && self.is_not_modifier(ev) {
            match ev.destructure() {
                EventSummary::Key(_, KeyCode::KEY_Z, _) => {
                    let _ = functions::run_command("wlr-which-key").await;
                }
                _ => {}
            }
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
            EventSummary::Key(_, KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT, value) => {
                self.shift_pressed = value == KeyEventType::PRESSED || value == KeyEventType::REPEAT;
            }
            EventSummary::Led(_, LedCode::LED_CAPSL, value) => {
                self.capslock_pressed = value == 1;
            }
            _ => {}
        }
    }

    fn all_modifiers_pressed(&mut self) -> bool {
        // set modifier keys here
        self.ctrl_pressed && self.shift_pressed
    }

    fn is_not_modifier(&mut self, ev: InputEvent) -> bool {
        // filter dud events
        if ev.event_type() == EventType::KEY && ev.value() == KeyEventType::PRESSED { } // pass
        else { return false };

        // modifier keys here and special keys to ignore
        match ev.destructure() {
            EventSummary::Key(_, KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT, _) => { return false } // alt key
            EventSummary::Key(_, KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL, _) => { return false } // ctl key
            EventSummary::Key(_, KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT, _) => { return false } // shift
            EventSummary::Key(_, KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA, _) => { return false } // meta
            EventSummary::Key(_, KeyCode::KEY_CAPSLOCK, _) => { return false } // caps
            _ => {}
        }

        true
    }
}