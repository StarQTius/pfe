#![no_std]
#![no_main]

use esp_idf_sys::{dilithium_reference_crypto_sign_verify, esp_task_wdt_init};
use log::info;
use rust_dilithium::{counter::SoftwareAesCounter, make_keys, sign, verify, SIGNATURE_SIZE};
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

    let (pk, sk) = make_keys::<HardwareAesCounter>(&true_random_seed()).unwrap();
    let mut signature = [0u8; SIGNATURE_SIZE];

    let software_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            chronometer.pause(|| {
                signature = sign::<HardwareAesCounter>(&true_random_seed(), &sk);
            });
            verify::<SoftwareAesCounter>(&true_random_seed(), &signature, &pk);
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
            chronometer.pause(|| {
                signature = sign::<HardwareAesCounter>(&true_random_seed(), &sk);
            });
            verify::<HardwareAesCounter>(&true_random_seed(), &signature, &pk);
        });
        chronometer.get()
    };
    info!(
        "Hardware perf: {hardware_time} for {TRIALS_NB} iterations ({}/it)",
        hardware_time / TRIALS_NB as u64
    );

    let reference_time = unsafe {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            let message = true_random_seed();

            chronometer.pause(|| {
                signature = sign::<HardwareAesCounter>(&true_random_seed(), &sk);
            });

            dilithium_reference_crypto_sign_verify(
                signature.as_ptr(),
                signature.len(),
                message.as_ptr(),
                message.len(),
                pk.as_ptr(),
            );
        });
        chronometer.get()
    };
    info!(
        "Reference perf: {reference_time} for {TRIALS_NB} iterations ({}/it)",
        reference_time / TRIALS_NB as u64
    );

    loop {}
}
