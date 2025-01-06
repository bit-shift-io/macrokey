use crate::util;

const TASK_ID: &str = "TEST";

pub async fn task() {
    info!("{}", TASK_ID);

    let device = util::get_device_by_name("AT Translated Set 2 keyboard").unwrap();
    //let device = Device::open("/dev/input/event10").unwrap();

    // event stream
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();
        info!("{:?}", ev);
    }
}