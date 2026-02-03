# flvparse

[![](https://github.com/koushiro/flvparse/actions/workflows/ci.yml/badge.svg)][actions]
[![](https://img.shields.io/docsrs/flvparse)][docs.rs]
[![](https://img.shields.io/crates/v/flvparse)][crates.io]
[![](https://img.shields.io/crates/l/flvparse)][crates.io]
[![](https://img.shields.io/crates/d/flvparse.svg)][crates.io]
[![](https://img.shields.io/badge/MSRV-1.85.0-green?logo=rust)][whatrustisit]

[actions]: https://github.com/koushiro/flvparse/actions
[docs.rs]: https://docs.rs/flvparse
[crates.io]: https://crates.io/crates/flvparse
[whatrustisit]: https://www.whatrustisit.com

A toy FLV format parsing library written in Rust with [nom](https://github.com/Geal/nom), mainly for learning `nom` (not production-ready).

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

```bash
cd cmd
cargo run -- --input ../assets/test.flv
```

```text
FLV File Header
┌─────────────────────────────┐
│ Field             Value     │
╞═════════════════════════════╡
│ Signature (3B)    46 4c 56  │
│ Version (1B)      1         │
│ Flags (1B)        0000 0101 │
│ DataOffset (4B)   9         │
└─────────────────────────────┘
Tag Summary
┌────────────────────────────────────────────────────────────────────────────┐
│ Total tag number   Script tag number   Video tag number   Audio tag number │
╞════════════════════════════════════════════════════════════════════════════╡
│ 13041              1                   4668               8372             │
└────────────────────────────────────────────────────────────────────────────┘
```

```bash
cd cmd
cargo run -- --input ../assets/test.flv -p
```

```text
FLV File Header
┌─────────────────────────────┐
│ Field             Value     │
╞═════════════════════════════╡
│ Signature (3B)    46 4c 56  │
│ Version (1B)      1         │
│ Flags (1B)        0000 0101 │
│ DataOffset (4B)   9         │
└─────────────────────────────┘
FLV File Body
┌───────────────────────────────────────────────────────────────────────┐
│ Index   TagType (1B)   DataSize (3B)   Timestamp (4B)   StreamID (3B) │
╞═══════════════════════════════════════════════════════════════════════╡
│ 1       Script         1030            0                0             │
│ 2       Video          48              0                0             │
│ 3       Audio          7               0                0             │
│ 4       Video          2831            0                0             │
│ 5       Video          104             41               0             │
| ...                                                                   |
│ 13038   Audio          15              194471           0             │
│ 13039   Audio          15              194494           0             │
│ 13040   Audio          15              194517           0             │
│ 13041   Video          5               194375           0             │
└───────────────────────────────────────────────────────────────────────┘
Tag Summary
┌────────────────────────────────────────────────────────────────────────────┐
│ Total tag number   Script tag number   Video tag number   Audio tag number │
╞════════════════════════════════════════════════════════════════════════════╡
│ 13041              1                   4668               8372             │
└────────────────────────────────────────────────────────────────────────────┘
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

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.
