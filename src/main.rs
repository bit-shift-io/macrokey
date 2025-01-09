#[macro_use]
extern crate log;

#[macro_use]
mod macros;

use tokio::{
    self,
    task::JoinSet,
};

mod utils;
mod tasks;
use tasks::*;
use utils::*;


#[tokio::main]
async fn main() {
    functions::init_logger();
    info!("== Start MacroKey ==");
    functions::check_permissions();
    functions::list_devices();
    info!("\n== Start Tasks ==");
    // tasks
    let mut set = JoinSet::new();
    //set.spawn(monitor::task("")); // log all events
    set.spawn(auto_repeat::task());
    set.spawn(remap::task());
    set.spawn(remote::task());
    set.spawn(virtual_device::task());
    set.join_all().await;
}