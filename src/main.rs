#[macro_use]
extern crate log;

use tokio::{
    self,
    task::JoinSet,
};

mod util;
mod tasks;
mod signals;
use tasks::*;


#[tokio::main]
async fn main() {
    util::init_logger();
    info!("== Start MacroKey ==");
    util::check_permissions();
    util::list_devices();
    
    // tasks
    let mut set = JoinSet::new();
    //set.spawn(monitor::task("")); // log all events
    set.spawn(auto_repeat::task());
    set.spawn(remote::task());
    set.spawn(virtual_device::task());
    set.join_all().await;
}