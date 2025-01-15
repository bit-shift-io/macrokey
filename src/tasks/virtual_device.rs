use evdev::{
    uinput::VirtualDeviceBuilder,
    AttributeSet,
    KeyCode, 
    RelativeAxisCode,
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
  
    // TODO: relativeaxiscode , add new()

    // all possible keys using an iterator method
    let keys = AttributeSet::from_iter((0..=0x2e7).map(KeyCode::new));
    
    // copy all axis
    let relative_axes = AttributeSet::from_iter([
        RelativeAxisCode::REL_X,
        RelativeAxisCode::REL_Y,
        RelativeAxisCode::REL_Z,
        RelativeAxisCode::REL_RX,
        RelativeAxisCode::REL_RY,
        RelativeAxisCode::REL_RZ,
        RelativeAxisCode::REL_HWHEEL,
        RelativeAxisCode::REL_DIAL,
        RelativeAxisCode::REL_WHEEL,
        RelativeAxisCode::REL_MISC,
        RelativeAxisCode::REL_RESERVED,
        RelativeAxisCode::REL_WHEEL_HI_RES,
        RelativeAxisCode::REL_HWHEEL_HI_RES,
    ]);

    // all possible relative axes using an inline closure
    //let relative_axes = AttributeSet::from_iter((0..=0x0c).map(|code| RelativeAxisCode(code)));

    // create a new device from an existing default
    let mut device = match VirtualDeviceBuilder::new() {
        Ok(builder) => {
            builder
                .name("macrokey virtual device")
                .with_keys(&keys).unwrap()
                .with_relative_axes(&relative_axes).unwrap()
                //.with_absolute_axis(&absolute_axis)?
                .build().unwrap()
        },
        Err(e) => {
            error!("{} Creation failed: {}", TASK_ID, e);
            return
        }
    };


    // display output device paths
    for path in device.enumerate_dev_nodes_blocking().unwrap() {
        let path = path.unwrap();
        info!("{}: {}", TASK_ID, path.display());
    }

    // get a lock on the receiver for the virtual device channel
    let mut rx = signals::get_virtual_device_rx().await;

    // handle the event in a loop
    while let Some(event) = rx.recv().await {
        device.emit(&[event.clone()]).unwrap();
    }
}