use sysinfo::{System, SystemExt};

use crate::u2client::types::HardwareStatus;

#[test]
fn test() {
    let sys = System::new_all();
    let res = HardwareStatus::new(&sys);
    println!("{:?}\n", res);
}
