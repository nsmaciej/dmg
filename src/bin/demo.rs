// Copyright 2017 dmg Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate dmg;

use std::env::args;
use std::path::PathBuf;

fn main() {
    let image = PathBuf::from(args().nth(1).expect("no image given"));
    let info = dmg::Attach::new(image)
        .force_readonly() // Force read-only
        .hidden() // Do not show up in Finder
        .with() // Detach when dropped
        .expect("could not attach");
    println!("Mount point: {:?}", info.mount_point);
    println!("Device:      {:?}", info.device);
}
