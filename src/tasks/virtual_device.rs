use evdev::{
    uinput::VirtualDeviceBuilder,
    AttributeSet,
    KeyCode,
};
use crate::signals;

const TASK_ID: &str = "VIRTUAL DEVICE";

/// Starts a virtual device and waits for events on the channel.
///
/// Creates a virtual device with all possible keys, including mouse buttons and gamepad keys.
/// Then, it waits for events on the channel and emits them to the virtual device.
pub async fn task() {
    info!("{}", TASK_ID);

    // all possible keys, inc mouse buttons & gamepad
    // let mut keys = AttributeSet::<KeyCode>::new();
    // for i in 0..=0x2e7 { keys.insert(KeyCode::new(i)); }

    // all possible keys using an iterator method
    let keys = AttributeSet::from_iter((0..=0x2e7).map(KeyCode::new));

    // create a new device from an existing default
    let mut device = VirtualDeviceBuilder::new().unwrap()
        .name("macrokey virtual device")
        .with_keys(&keys).unwrap()
        .build()
        .unwrap();

    // display output device paths
    for path in device.enumerate_dev_nodes_blocking().unwrap() {
        let path = path.unwrap();
        println!("{}: {}", TASK_ID, path.display());
    }

    // get a lock on the receiver for the virtual device channel
    let mut rx = signals::get_virtual_device_rx().await;

    // handle the event in a loop
    while let Some(event) = rx.recv().await {
        device.emit(&[*event]).unwrap();
        info!("{} code: {} pressed: {}", TASK_ID, event.destructure().0.0, event.destructure().1);
    }
}