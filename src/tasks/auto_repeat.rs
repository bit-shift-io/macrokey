use evdev::{
    KeyCode,
    InputEvent,
    EventType,
};
use tokio::time::{
    sleep,
    Duration
};
use crate::{
    key_event_type::KeyEventType, 
    signals,
};

const TASK_ID: &str = "AUTO REPEAT";

pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        //task_test_send().await;

        info!("{} error, retry in 60s", TASK_ID);
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}


pub async fn task_test_send() {
    let tx = signals::get_virtual_device_tx().await;

    loop {
        let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::PRESSED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(2)).await;

        let ie = InputEvent::new_now(EventType::KEY.0, KeyCode::KEY_B.0, KeyEventType::RELEASED.into());
        tx.send(ie).await.unwrap();
        sleep(Duration::from_secs(2)).await;
    }
}