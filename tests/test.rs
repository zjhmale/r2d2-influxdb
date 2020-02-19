extern crate r2d2_influxdb;

use std::sync::mpsc;
use std::thread;

use r2d2_influxdb::{r2d2, InfluxDBConnectionManager, SimpleInfo};

#[test]
fn test_basic() {
    let info = SimpleInfo {
        url: "http://localhost:8086".into(),
        database: "test".into(),
    };
    let manager = InfluxDBConnectionManager::new(info);
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();

    let (s1, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();

    let pool1 = pool.clone();
    let t1 = thread::spawn(move || {
        let conn = pool1.get().unwrap();
        s1.send(()).unwrap();
        r2.recv().unwrap();
        drop(conn);
    });

    let pool2 = pool.clone();
    let t2 = thread::spawn(move || {
        let conn = pool2.get().unwrap();
        s2.send(()).unwrap();
        r1.recv().unwrap();
        drop(conn);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    pool.get().unwrap();
}

#[test]
fn test_is_valid() {
    let info = SimpleInfo {
        url: "http://localhost:8086".into(),
        database: "test".into(),
    };
    let manager = InfluxDBConnectionManager::new(info);
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .test_on_check_out(true)
        .build(manager)
        .unwrap();

    let conn = pool.get().unwrap();
    assert!(conn.ping().is_ok());
}
