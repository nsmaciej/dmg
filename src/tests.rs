// Copyright 2022 dmg Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fs::File;

use super::*;

static SAMPLE_DIR_PATH: &str = "test";
static SAMPLE_FILE_NAME: &str = "SAMPLE";
const ERRRNO_EROFS: i32 = 30;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

// #[test]
// fn create_overwrite() {
//     init();
//     let image = FromFolder::new(SAMPLE_DIR_PATH)
//         .volume_name("create_overwrite")
//         .create_temp()
//         .expect("error creating");

//     assert!(FromFolder::new(SAMPLE_DIR_PATH)
//         .volume_name("create_overwrite")
//         .create(&image)
//         .is_err());
//     assert!(FromFolder::new(SAMPLE_DIR_PATH)
//         .volume_name("create_overwrite")
//         .overwrite()
//         .create(&image)
//         .is_ok());
// }
#[test]
fn detach_on_drop() {
    init();
    let image = FromFolder::new(SAMPLE_DIR_PATH)
        .volume_name("detach_on_drop")
        .create_temp()
        .expect("error creating");

    let mount_point;
    {
        let info = Attach::new(&image)
            .mount_temp()
            .hidden()
            .with()
            .expect("error attaching");
        mount_point = info.mount_point.clone();
    }
    assert!(!mount_point.exists());
}

#[test]
fn force_readonly() {
    init();
    let image = FromFolder::new(SAMPLE_DIR_PATH)
        .volume_name("force_readonly")
        .create_temp()
        .expect("error creating");

    let info = Attach::new(&image)
        .mount_temp()
        .force_readonly()
        .hidden()
        .with()
        .expect("error attaching");

    let err =
        File::create(info.mount_point.join(SAMPLE_FILE_NAME)).expect_err("create should fail");
    assert_eq!(err.raw_os_error(), Some(ERRRNO_EROFS));
}
