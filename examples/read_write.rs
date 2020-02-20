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
