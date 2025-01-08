use evdev::{
        KeyCode,
        KeyEvent,
    };
use tokio::time::{
        sleep,
        Duration
    };
use crate::signals;

const TASK_ID: &str = "DEFAULT";

pub async fn task() {
    info!("{}", TASK_ID);

    loop {
        task_test_send().await;

        info!("{} error, retry in 60s", TASK_ID);
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}


pub async fn task_test_send() {
    let tx = signals::get_virtual_device_tx().await;

    // loop {
    //     let event = KeyEvent::new(KeyCode::KEY_B, 1);
    //     tx.send(event).await.unwrap();
    //     sleep(Duration::from_secs(2)).await;

    //     let event = KeyEvent::new(KeyCode::KEY_B, 0);
    //     tx.send(event).await.unwrap();
    //     sleep(Duration::from_secs(2)).await;
    // }
}