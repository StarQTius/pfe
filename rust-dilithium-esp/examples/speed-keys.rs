#![no_std]
#![no_main]

use esp_idf_sys::{dilithium_reference_crypto_sign_keypair, esp_task_wdt_init};
use log::info;
use rust_dilithium::{counter::SoftwareAesCounter, make_keys, PUBLIC_KEY_SIZE, SECRET_KEY_SIZE};
use rust_dilithium_esp::{true_random_seed, HardwareAesCounter, Timer};

type Chronometer = Timer<0, 0>;

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    const TRIALS_NB: usize = 1000;

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    unsafe {
        esp_task_wdt_init(86400, false);
    }

    info!(file!());

    let software_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            make_keys::<SoftwareAesCounter>(&true_random_seed());
        });
        chronometer.get()
    };
    info!(
        "Software perf: {software_time} for {TRIALS_NB} iterations ({}/it)",
        software_time / TRIALS_NB as u64
    );

    let hardware_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            make_keys::<HardwareAesCounter>(&true_random_seed());
        });
        chronometer.get()
    };
    info!(
        "Hardware perf: {hardware_time} for {TRIALS_NB} iterations ({}/it)",
        hardware_time / TRIALS_NB as u64
    );

    let reference_time = unsafe {
        let mut pk = [0u8; PUBLIC_KEY_SIZE];
        let mut sk = [0u8; SECRET_KEY_SIZE];

        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            dilithium_reference_crypto_sign_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
        });
        chronometer.get()
    };
    info!(
        "Reference perf: {reference_time} for {TRIALS_NB} iterations ({}/it)",
        reference_time / TRIALS_NB as u64
    );

    loop {}
}
