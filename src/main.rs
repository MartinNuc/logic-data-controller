extern crate sysfs_gpio;
mod table_controller;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};
use table_controller::{ControlPins, TableController};

const FRAME_LENGTH: u128 = 1000;

fn main() {
    let (tx, rx) = channel();
    let table = TableController::new(ControlPins {
        up_motor_pin: 22,
        up_controller_pin: 24,
        down_motor_pin: 27,
        down_controller_pin: 23,
        signal_motor_pin: 17,
        signal_controller_pin: 25,
    });

    start_interrupt_thread(tx, &table);
    start_main_loop(rx, table);
}

fn start_main_loop(rx: Receiver<u128>, mut table: TableController) {
    let mut last_time_checked: u128 = micros();
    loop {
        if let Ok(val) = rx.try_recv() {
            last_time_checked = val;
        }
        let now = micros();
        if now - last_time_checked >= FRAME_LENGTH {
            if now - last_time_checked > 2 * FRAME_LENGTH {
                println!("Missed the frame 😥 by {}", now - last_time_checked);
            }
            last_time_checked = now;
            table.tick().unwrap();
        }
    }
}

fn start_interrupt_thread(tx: Sender<u128>, table: &TableController) {
    let mut poller = table.wait_for_interrupt().unwrap();
    std::thread::spawn(move || loop {
        if let Ok(_value) = poller.poll(1000) {
            let last_time_checked = micros();
            tx.send(last_time_checked).unwrap();
        }
    });
}

fn micros() -> u128 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_micros()
}
