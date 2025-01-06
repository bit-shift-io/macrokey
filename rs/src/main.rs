// Import the log crate
#[macro_use]
extern crate log;

extern crate uinput_tokio;
use env_logger::Builder;
use std::{io::Write, thread::spawn};
use std::env;
use tokio::{self, task::JoinSet};
//use futures::future::join_all;
mod event_device;
mod util;
mod tasks;
use tasks::*;


#[tokio::main]
async fn main() {
    init_logger();
    info!("== Start MacroKey ==");
    util::check_permissions();
    util::list_devices();
    

    //let event_devices = event_device::get_input_devices().await;

    // set version
    let mut set = JoinSet::new();
    set.spawn(default::task());
    set.spawn(test::task());
    set.join_all().await;
}

fn init_logger() {
    env::set_var("RUST_LOG", "info");
    Builder::from_default_env()
        .format(|buf, record| {
            //let level = record.level();
            let message = record.args();
            writeln!(buf, "{}", message)
        })
        .init();
    // let mut builder = Builder::from_default_env();
    // builder.format_timestamp(None);
    // builder.format_module_path(false);
    // builder.format_level(true);
    // builder.init();
}