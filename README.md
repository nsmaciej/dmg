# dmg

Simple attaching/detaching of macOS disk images.

![Build Status](https://github.com/maciej-irl/dmg/actions/workflows/ci.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/dmg.svg)](https://crates.io/crates/dmg)
[![Docs](https://docs.rs/dmg/badge.svg)](https://docs.rs/dmg)

## Examples

### Attaching

Attach a disk image until dropped:

```rust,no_run
let info = dmg::Attach::new("Test.dmg")
    .with()
    .expect("could not attach");
println!("Mounted at {:?}", info.mount_point);
// Detched when 'info' dropped.
```

If you prefer to handle detaching yourself simply use [`attach()`](https://docs.rs/dmg/latest/dmg/struct.Attach.html):

```rust,no_run
let info = dmg::Attach::new("Test.dmg")
    .attach()
    .expect("could not attach");
println!("Device node {:?}", info.device);
info.detach().expect("could not detach");
```

### Creation

Create a new disk image:

```rust,no_run
let image_path = dmg::FromFolder::new("test")
    .volume_name("My Volume") // You can give a volume name.
    .create("Test.dmg")
    .expect("create failed");
println!("Devide path {image_path:?}");
```

### Detaching

If you know the device node or mount point, you can detach it directly:

```rust,no_run
dmg::detach("/Volumes/Test", false) // Do not force detach.
    .expect("could not detach"); 
```

For more examples see [`src/tests.rs`][1] and [`src/bin/demo.rs`][2]

[1]: https://github.com/maciej-irl/dmg/blob/master/src/tests.rs
[2]: https://github.com/maciej-irl/dmg/blob/master/bin/demo.rs

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
