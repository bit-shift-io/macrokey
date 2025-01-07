use crate::util;

pub async fn task(name: &str, device_name: &str) {
    info!("{}", name);

    let device = util::get_device_by_name(device_name).unwrap();
    //let device = Device::open("/dev/input/event10").unwrap();

    // event stream
    let mut events = device.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();
        //info!("{:?}", ev);
    }
}