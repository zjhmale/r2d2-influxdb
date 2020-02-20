<h1 align="center">r2d2-influxdb</h1>
<p align="center">
    Rust Connection Pool for InfluxDB
</p>

[![travis-ci.org](https://travis-ci.org/zjhmale/r2d2-influxdb.svg)](https://travis-ci.org/zjhmale/r2d2-influxdb)
[![crates.io](https://img.shields.io/crates/v/r2d2-influxdb.svg)](https://crates.io/crates/r2d2-influxdb)
[![Documentation](https://docs.rs/r2d2-influxdb/badge.svg)](https://docs.rs/crate/r2d2-influxdb)

[influxdb-rust](https://github.com/Empty2k12/influxdb-rust) support library for the [r2d2](https://github.com/sfackler/r2d2) connection pool protocol.

# Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
r2d2_influxdb = "0.1.0"
```

Then check out the examples below.

# Examples

See [examples](examples) for runnable examples.

## Quick start

With `pool.get()`, we can basically get a synchronized wrapper of `influxdb::Client` which inherit all the
magic without `async/await` only.

```rust
extern crate r2d2_influxdb;

use r2d2_influxdb::{influxdb, r2d2, AuthInfo, InfluxDBConnectionManager};
use std::thread;

use influxdb::{Query, Timestamp};

fn main() {
    let info = AuthInfo {
        url: "http://localhost:8086".into(),
        database: "weather".into(),
        username: "root".into(),
        password: "root".into(),
    };
    let manager = InfluxDBConnectionManager::new(info);
    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let mut write_handles = vec![];
    let mut read_handles = vec![];

    for _i in 0..10i32 {
        let pool = pool.clone();
        let write_query =
            Query::write_query(Timestamp::Now, "weather").add_field("temperature", _i);
        write_handles.push(thread::spawn(move || {
            let conn = pool.get().unwrap();
            let write_result = conn.query(&write_query);
            println!("{:?}", write_result);
        }));
    }

    for h in write_handles {
        h.join().unwrap();
    }

    for _i in 0..10i32 {
        let pool = pool.clone();
        let read_query = Query::raw_read_query("SELECT * FROM weather");
        read_handles.push(thread::spawn(move || {
            let conn = pool.get().unwrap();
            let read_result = conn.query(&read_query);
            println!("{:?}", read_result);
        }));
    }

    for h in read_handles {
        h.join().unwrap();
    }
}
```

## Acknowledgement

This project is heavily inspired by [r2d2-redis](https://github.com/sorccu/r2d2-redis).

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

