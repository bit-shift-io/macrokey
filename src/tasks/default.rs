use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, KeyCode};


const TASK_ID: &str = "DEFAULT";

pub async fn task() {
    info!("{}", TASK_ID);

    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_DPAD_UP);

    // let test = AttributeSet::from_iter([
    //     RelativeAxisCode::REL_X,
    //     RelativeAxisCode::REL_Y,
    //     RelativeAxisCode::REL_WHEEL,
    //     RelativeAxisCode::REL_HWHEEL,
    // ]);

    // create a new device from an existing default
    let mut device = VirtualDeviceBuilder::new().unwrap()
        .name("Fake Keyboard")
        .with_keys(&keys).unwrap()
        .build()
        .unwrap();

}