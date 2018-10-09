# flvparser

[![Build Status](https://travis-ci.org/koushiro/flvparser.svg?branch=master)](https://travis-ci.org/koushiro/flvparser)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/koushiro/flvparser?branch=master&svg=true)](https://ci.appveyor.com/project/koushiro/flvparser)
[![GitHub License](https://img.shields.io/github/license/koushiro/flvparser.svg)](https://github.com/koushiro/flvparser/blob/master/LICENSE)

A FLV file parser written in Rust with [nom](https://github.com/Geal/nom).

## Usage

### Simple example

```rust
extern crate flvparser;
use flvparser::{FLVFile, parse_flv_file};

fn main() {
    let flv_file = FLVParser::parse(include_bytes!("assets/test.flv")).unwrap();
    // ...
}
```

For a detailed example, see [main.rs](src/main.rs)

![](assets/usage.gif)

### Related structure

```bash
FLVFile
├──FLVFileHeader
└──FLVFileBody
   ├──u32 -- first previous tag size
   └──Vec<(FLVTag. u32)>
    
FLVTag
├──FLVTagHeader
└──FLVTagData

FLVTagData
└──ScriptTag/VideoTag/AudioTag

```

## License

MIT