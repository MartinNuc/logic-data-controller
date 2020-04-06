extern crate crossbeam_channel;
extern crate sysfs_gpio;

mod table_controller;
mod web_server;

use crate::table_controller::{ControlPins, TableController};
use crate::web_server::TableInfo;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};

const FRAME_LENGTH: u128 = 1000;

fn main() {
    let (tx_last_time_checked, rx_last_time_checked) = bounded(1);
    let (tx_table_info_request, rx_table_info_request) = bounded(1);
    let (tx_table_info_response, rx_table_info_response) = bounded(1);
    let (tx_set_target_height, rx_set_target_height) = bounded(1);

    let table = TableController::new(
        // TODO pass pin numbers as env var
        ControlPins {
            up_motor_pin: 22,
            up_controller_pin: 24,
            down_motor_pin: 27,
            down_controller_pin: 23,
            signal_motor_pin: 17,
            signal_controller_pin: 25,
        },
        tx_table_info_response,
        rx_table_info_request,
        rx_set_target_height,
    );

    start_web_server_thread(
        tx_set_target_height,
        tx_table_info_request,
        rx_table_info_response,
    );
    start_interrupt_thread(tx_last_time_checked, &table);
    start_main_loop(rx_last_time_checked, table);
}

fn start_web_server_thread(
    tx_set_target_height: Sender<i32>,
    tx_table_info_request: Sender<()>,
    rx_table_info_response: Receiver<TableInfo>,
) {
    std::thread::spawn(|| {
        crate::web_server::start_server(
            tx_set_target_height,
            tx_table_info_request,
            rx_table_info_response,
        );
    });
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
            let last_time_checked = micros() - FRAME_LENGTH / 2;
            tx.send(last_time_checked).unwrap();
        }
    });
}

fn micros() -> u128 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_micros()
}
