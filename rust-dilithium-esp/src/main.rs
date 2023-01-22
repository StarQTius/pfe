#![no_std]
#![no_main]

use core::iter::repeat;
use esp_idf_sys as _;
use log::info;
use rust_dilithium::{counter::SoftwareAesCounter, make_keys, sign, verify};

#[no_mangle]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("START");

    let (pk, sk) = make_keys::<SoftwareAesCounter>(repeat(0)).unwrap();
    let signature = sign::<SoftwareAesCounter>(&[0u8; 32], &sk);

    info!("{signature:?}");

    if verify::<SoftwareAesCounter>(&[0u8; 32], &signature, &pk) {
        info!("VERIFIED !!")
    }
}
