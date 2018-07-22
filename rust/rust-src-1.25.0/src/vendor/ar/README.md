# rust-ar

A rust library for encoding/decoding Unix archive (.a) files.

Documentation: https://mdsteele.github.io/rust-ar/

## Overview

The `ar` crate is a pure Rust implementation of a
[Unix archive file](https://en.wikipedia.org/wiki/Ar_(Unix)) reader and writer.
This library provides a streaming interface, similar to that of the
[`tar`](https://crates.io/crates/tar) crate, that avoids having to ever load a
full archive entry into memory.

## License

rust-ar is made available under the
[MIT License](http://spdx.org/licenses/MIT.html).
