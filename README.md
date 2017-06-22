# dmg

Simple attaching/detaching of macOS disk images.

[![Build Status](https://travis-ci.org/mgoszcz2/dmg.svg?branch=master)](https://travis-ci.org/mgoszcz2/dmg)
[![crates.io](https://img.shields.io/crates/v/dmg.svg)](https://crates.io/crates/dmg)
[![Docs](https://docs.rs/dmg/badge.svg)](https://docs.rs/dmg)

## Example

Attach a disk image until dropped:

```rust
use dmg::Attach;
let info = Attach::new("Test.dmg").with().expect("could not attach");
println!("Mounted at {:?}", info.mount_point);
// Detched when 'info' dropped
```

If you prefer to handle detaching yourself simply use `attach()`:

```rust
use dmg::Attach;
let info = Attach::new("Test.dmg").attach().expect("could not attach");
println!("Device node {:?}", info.device);
info.detach().expect("could not detach"); // There is also .force_detach()
```

If you know the device node or mount point, you can detach it like this too:

```rust
use dmg;
dmg::detach("/Volumes/Test", false).expect("could not detach"); // Do not force detach
```

For more examples see [`src/tests.rs`][1] and [`src/bin/demo.rs`][2]

[1]: https://github.com/mgoszcz2/dmg/blob/master/src/tests.rs
[2]: https://github.com/mgoszcz2/dmg/blob/master/src/bin/demo.rs

## Testing

To create `Test.dmg` run:

```bash
./create_dmg.sh
```

This will create a read-write .dmg file containg a single file called `SAMPLE`.

`hdiutil` doesn not like attaching and detaching the same file concurrently, so test using:

```bash
cargo test -- --test-threads 1
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
