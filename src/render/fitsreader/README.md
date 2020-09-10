
# FITSReader written in pure Rust using [nom](https://github.com/Geal/nom)

This crate is under heavy development, it was initiated for reading fits HiPS tile, i.e. generated from hipsgen and therefore taking into account that a fits tile:

* Contains all its data the primary unit
* Does not use any WCS

## What is supported ?
The read of a primary unit, i.e. the primary header and data unit
The extensions are not supported

## Exemple

```rust
use std::fs::File;
use fitsreader::{Fits, DataType};

let f  = File::open("misc/Npix208.fits").unwrap();
let bytes: Result<Vec<_>, _> =  f.bytes().collect();
let buf  =  bytes.unwrap();
let Fits { data, .. } =  Fits::from_bytes_slice(&buf).unwrap();

match data {
    DataType::F32(v) => {
        println!("{:?}", v);
    },
    _ => unreachable!()
};
```

To run the tests:
``
cargo test
``