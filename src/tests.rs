// Copyright 2017 dmg Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fs::File;

use super::*;

static SAMPLE_IMAGE_PATH: &str = "Test.dmg";
static SAMPLE_FILE_NAME: &str = "SAMPLE";
const ERRRNO_EROFS: i32 = 30;

macro_rules! logger {
    () => {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}

#[test]
fn detach_on_drop() {
    logger!();
    let mount_point;
    {
        let info = Attach::new(SAMPLE_IMAGE_PATH)
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
    logger!();
    let info = Attach::new(SAMPLE_IMAGE_PATH)
        .mount_temp()
        .force_readonly()
        .hidden()
        .with()
        .expect("error attaching");

    let err = File::create(info.mount_point.join(SAMPLE_FILE_NAME))
        .expect_err("create should fail");
    assert_eq!(err.raw_os_error(), Some(ERRRNO_EROFS));
}
