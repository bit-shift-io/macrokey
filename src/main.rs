// Import the log crate
#[macro_use]
extern crate log;

use tokio::{self, task::JoinSet};
mod util;
mod tasks;
use tasks::*;


#[tokio::main]
async fn main() {
    init_logger();
    info!("== Start MacroKey ==");
    util::check_permissions();
    util::list_devices();
    
    // tasks
    let mut set = JoinSet::new();
    //set.spawn(monitor::task("")); // log all devices
    set.spawn(default::task());
    //set.spawn(template::task("template", "AT Translated Set 2 keyboard")); // test proxy fn
    set.spawn(remote::task());
    set.join_all().await;
}


fn init_logger() {
    use std::io::Write;
    use std::env;
    use env_logger::Builder;
    env::set_var("RUST_LOG", "info");
    Builder::from_default_env()
        .format(|buf, record| {
            //let level = record.level();
            let message = record.args();
            writeln!(buf, "{}", message)
        })
        .init();
}