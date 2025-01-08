use evdev::{
    uinput::VirtualDeviceBuilder,
    AttributeSet,
    EventType,
    InputEvent,
    KeyCode,
    KeyEvent,
};
use crate::signals;


const TASK_ID: &str = "VIRTUAL DEVICE";

pub async fn task() {
    info!("{}", TASK_ID);

    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_DPAD_UP);
    keys.insert(KeyCode::KEY_B);

    // alternate method of creating an AttributeSet
    // let test = AttributeSet::from_iter([
    //     KeyCode::BTN_DPAD_UP,
    //     KeyCode::BTN_DPAD_DOWN,
    // ]);

    // create a new device from an existing default
    let mut device = VirtualDeviceBuilder::new().unwrap()
        .name("Repeat Keyboard")
        .with_keys(&keys).unwrap()
        .build()
        .unwrap();

    // display output device paths
    for path in device.enumerate_dev_nodes_blocking().unwrap() {
        let path = path.unwrap();
        println!("Virtual device: {}", path.display());
    }

    // get a lock on the receiver for the virtual device channel
    let mut rx = signals::get_virtual_device_rx().await;

    // handle the event in a loop
    while let Some(event) = rx.recv().await {
        device.emit(&[*event]).unwrap();
        info!("Sent code: {} pressed: {}", event.destructure().0.0, event.destructure().1);
    }
}