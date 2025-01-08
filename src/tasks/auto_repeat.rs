use evdev::{
    uinput::VirtualDeviceBuilder,
    AttributeSet,
    EventType,
    InputEvent,
    KeyCode,
    KeyEvent,
};
use tokio::{
    sync::mpsc,
    task::JoinSet,
    time::{
        sleep,
        Duration
    },
};


const TASK_ID: &str = "DEFAULT";

pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        // virtual device channel to communicate
        let (tx, mut rx) = mpsc::channel::<KeyEvent>(32);

        let mut set = JoinSet::new();
        set.spawn(task_virtual_device(rx));
        //set.spawn(task_test_send(tx.clone()));
        set.join_all().await;

        info!("{} error, retry in 60s", TASK_ID);
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}


pub async fn task_test_send(mut tx: mpsc::Sender<KeyEvent>) {
    loop {
        let event = KeyEvent::new(KeyCode::BTN_DPAD_UP, 1);
        tx.send(event).await.unwrap();
        sleep(Duration::from_secs(2)).await;

        let event = KeyEvent::new(KeyCode::BTN_DPAD_UP, 0);
        tx.send(event).await.unwrap();
        sleep(Duration::from_secs(2)).await;
    }
}


async fn task_virtual_device(mut rx: mpsc::Receiver<KeyEvent>) {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_DPAD_UP);

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

    loop {
        let event = rx.recv().await.unwrap();
        device.emit(&[*event]).unwrap();
        info!("Sent code: {} pressed: {}", event.destructure().0.0, event.destructure().1);
    }
}