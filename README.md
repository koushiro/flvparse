# flvparser

[![Build Status](https://travis-ci.org/koushiro/flvparser.svg?branch=master)](https://travis-ci.org/koushiro/flvparser)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/koushiro/flvparser?branch=master&svg=true)](https://ci.appveyor.com/project/koushiro/flvparser)
[![GitHub License](https://img.shields.io/github/license/koushiro/flvparser.svg)](https://github.com/koushiro/flvparser/blob/master/LICENSE)

A FLV file parser written in Rust with [nom](https://github.com/Geal/nom).

## Usage

### Simple example

```toml
[dependencies]
flvparse = { git = "https://github.com/koushiro/flvparser" }
```

```rust
fn main() {
    let flv = flvparse::parse(include_bytes!("assets/test.flv")).unwrap();
    // ...
}
```

For a detailed example, see [example](src/cli.rs).

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
   └──Vec<(FlvTag. u32)>
    
FlvTag
├──FlvTagHeader
└──FlvTagData

FlvTagData
└──ScriptTag/VideoTag/AudioTag
```

## License

MIT
