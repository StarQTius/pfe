#![no_std]
#![no_main]

use esp_idf_sys::{esp_task_wdt_init, vTaskDelay};
use log::info;
use rust_dilithium::{make_keys, sign, SEED_SIZE};
use rust_dilithium_esp::{true_random_seed, HardwareAesCounter};

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    unsafe {
        esp_task_wdt_init(86400, false);
    }

    info!(file!());

    let (_, sk) = make_keys::<HardwareAesCounter>(&[0u8; SEED_SIZE / 2]).unwrap();

    loop {
        let message = true_random_seed();
        let signature = sign::<HardwareAesCounter>(&message, &sk);
        unsafe {
            info!("{message:?}");
            info!("{signature:?}");
            vTaskDelay(100);
        }
    }
}
