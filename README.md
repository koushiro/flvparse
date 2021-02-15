# flvparse

[![ga-svg]][ga-url]
[![crates-svg]][crates-url]
[![docs-svg]][docs-url]
[![codecov-svg]][codecov-url]
[![deps-svg]][deps-url]

[ga-svg]: https://github.com/koushiro/flvparse/workflows/build/badge.svg
[ga-url]: https://github.com/koushiro/flvparse/actions
[crates-svg]: https://img.shields.io/crates/v/flvparse
[crates-url]: https://crates.io/crates/flvparse
[docs-svg]: https://docs.rs/flvparse/badge.svg
[docs-url]: https://docs.rs/flvparse
[codecov-svg]: https://img.shields.io/codecov/c/github/koushiro/flvparse
[codecov-url]: https://codecov.io/gh/koushiro/flvparse
[deps-svg]: https://deps.rs/repo/github/koushiro/flvparse/status.svg
[deps-url]: https://deps.rs/repo/github/koushiro/flvparse

A FLV format parsing library written in Rust with [nom](https://github.com/Geal/nom).

## Usage

### Quick start

```toml
[dependencies]
flvparse = "0.1"
```

```rust
fn main() {
    let bytes = include_bytes!("assets/test.flv");
    let flv = flvparse::FlvFile::parse(bytes).unwrap();
    // ...
}
```

### Example

See [example](cmd/src/main.rs) for details.

```
cd cmd
cargo run -- --input ../assets/test.flv

+-----------------+-----------+
| FLV File Header |           |
+-----------------+-----------+
| Signature (3B)  | 46 4c 56  |
| Version (1B)    | 1         |
| Flags (1B)      | 0000 0101 |
| DataOffset (4B) | 9         |
+-----------------+-----------+
+------------------+-------------------+------------------+------------------+
| Total tag number | Script tag number | Video tag number | Audio tag number |
+------------------+-------------------+------------------+------------------+
| 13041            | 1                 | 4668             | 8372             |
+------------------+-------------------+------------------+------------------+
```

```
cd cmd
cargo run -- --input ../assets/test.flv -p
+-----------------+-----------+
| FLV File Header |           |
+-----------------+-----------+
| Signature (3B)  | 46 4c 56  |
| Version (1B)    | 1         |
| Flags (1B)      | 0000 0101 |
| DataOffset (4B) | 9         |
+-----------------+-----------+
+---------------+--------------+---------------+----------------+---------------+
| FLV File Body |              |               |                |               |
+---------------+--------------+---------------+----------------+---------------+
| Index         | TagType (1B) | DataSize (3B) | Timestamp (4B) | StreamID (3B) |
| 1             | Script       | 1030          | 0              | 0             |
| 2             | Video        | 48            | 0              | 0             |
| 3             | Audio        | 7             | 0              | 0             |
| 4             | Video        | 2831          | 0              | 0             |
| ...                                                                           |
| 13039         | Audio        | 15            | 194494         | 0             |
| 13040         | Audio        | 15            | 194517         | 0             |
| 13041         | Video        | 5             | 194375         | 0             |
+---------------+--------------+---------------+----------------+---------------+
+------------------+-------------------+------------------+------------------+
| Total tag number | Script tag number | Video tag number | Audio tag number |
+------------------+-------------------+------------------+------------------+
| 13041            | 1                 | 4668             | 8372             |
+------------------+-------------------+------------------+------------------+
```

### Related structure

```
FlvFile
├──FlvFileHeader
└──FlvFileBody
   ├──u32 -- first previous tag size
   └──Vec<(FlvTag, u32)>
    
FlvTag
├──FlvTagHeader
└──FlvTagData

FlvTagData
└──ScriptTag/VideoTag/AudioTag
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
