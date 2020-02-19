extern crate r2d2_influxdb;

use r2d2_influxdb::{r2d2, InfluxDBConnectionManager, SimpleInfo};
use std::thread;

fn main() {
    let info = SimpleInfo {
        url: "http://localhost:8086".into(),
        database: "test".into(),
    };
    let manager = InfluxDBConnectionManager::new(info);
    let pool = r2d2::Pool::builder().build(manager).unwrap();

    let mut handles = vec![];

    for _i in 0..10i32 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let conn = pool.get().unwrap();
            let reply = conn.ping();
            println!("{:?}", reply);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
