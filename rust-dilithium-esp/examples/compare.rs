#![no_std]
#![no_main]

use core::iter::zip;
use log::info;
use rust_dilithium_esp::{compute_hardware, compute_software, true_random_seed};

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let msg = true_random_seed();
    let seed = true_random_seed();

    let sw_signature = compute_software(&msg, &seed).unwrap();
    let hw_signature = compute_hardware(&msg, &seed).unwrap();

    if sw_signature == hw_signature {
        info!("VERIFIED");
    } else {
        info!("ERROR");
        for (sw_byte, hw_byte) in zip(sw_signature.iter(), hw_signature.iter()) {
            if sw_byte == hw_byte {
                info!("{} == {}", sw_byte, hw_byte);
            } else {
                info!("{} != {}", sw_byte, hw_byte);
            }
        }
    }

    loop {}
}
