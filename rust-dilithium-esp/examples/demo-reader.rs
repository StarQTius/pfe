#![no_std]
#![no_main]

use core::iter::repeat;
use esp_idf_sys::{esp_task_wdt_init, getchar};
use log::info;
use rust_dilithium::{make_keys, verify, SEED_SIZE, SIGNATURE_SIZE};
use rust_dilithium_esp::HardwareAesCounter;

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    unsafe {
        esp_task_wdt_init(86400, false);
    }

    info!(file!());

    let (pk, _) = make_keys::<HardwareAesCounter>(&[0u8; SEED_SIZE / 2]).unwrap();
    let mut buf = [0u8; SEED_SIZE / 2 + SIGNATURE_SIZE];

    loop {
        unsafe {
            let mut counter = 4;
            while counter > 0 {
                counter = match getchar() {
                    0xff => counter - 1,
                    -1 => counter,
                    _ => 4,
                }
            }

            for val in buf.iter_mut() {
                *val = repeat(0)
                    .find_map(|_| match getchar() {
                        -1 => None,
                        n => Some(n as u8),
                    })
                    .unwrap();
            }
        }
        let message = &buf[..SEED_SIZE / 2];
        let signature = buf[SEED_SIZE / 2..].try_into().unwrap();
        if verify::<HardwareAesCounter>(message, &signature, &pk) {
            info!("verified !");
        } else {
            info!("rejected !");
        }
    }
}
