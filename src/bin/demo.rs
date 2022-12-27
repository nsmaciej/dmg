// Copyright 2022 dmg Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

fn main() {
    let temp_image = dmg::FromFolder::new("test")
        .volume_name("demo")
        .create_temp()
        .expect("could not create");

    println!("Disk image:  {:?}", temp_image.as_os_str());

    let info = dmg::Attach::new(&temp_image)
        .force_readonly() // Force read-only
        .hidden() // Do not show up in Finder
        .with() // Detach when dropped
        .expect("could not attach");

    println!("Mount point: {:?}", info.mount_point);
    println!("Device:      {:?}", info.device);
}
