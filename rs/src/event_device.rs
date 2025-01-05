use log::info;
use tokio::fs;



pub struct EventDevice {
    filename: String,
}

async fn is_event_device(entry: &fs::DirEntry) -> bool {
    let file_name_osstr = entry.file_name();
    let file_name = file_name_osstr.to_str().unwrap();
    if !file_name.starts_with("by-id") && !file_name.starts_with("by-path") {
        return true;
    }
    // old cpp code, this excludes mice and other devices not startign with event
    // if let Some(file_name) = entry.file_name().to_str() {
    //     return file_name.starts_with("event");
    // }
    false
}


pub async fn get_system_devices() -> Vec<EventDevice> {
    let mut list = Vec::new();

    let entries = match fs::read_dir("/dev/input").await {
        Ok(entries) => entries,
        Err(_) => return list,
    };

    info!("System devices:");

    let mut entries = entries;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if is_event_device(&entry).await {
            let path = entry.path();
            let filename = path.to_string_lossy().into_owned();
            list.push(EventDevice { filename: filename.clone() });
            info!("{}", filename);
        }
    }

    list
}