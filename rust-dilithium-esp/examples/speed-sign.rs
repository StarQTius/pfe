#![no_std]
#![no_main]

use esp_idf_sys::{dilithium_reference_crypto_sign_signature, esp_task_wdt_init};
use log::info;
use rust_dilithium::{counter::SoftwareAesCounter, make_keys, sign, SEED_SIZE, SIGNATURE_SIZE};
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

    let (_, sk) = make_keys::<HardwareAesCounter>(&true_random_seed()).unwrap();

    let software_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            sign::<SoftwareAesCounter>(&true_random_seed(), &sk);
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
            sign::<HardwareAesCounter>(&true_random_seed(), &sk);
        });
        chronometer.get()
    };
    info!(
        "Hardware perf: {hardware_time} for {TRIALS_NB} iterations ({}/it)",
        hardware_time / TRIALS_NB as u64
    );

    let reference_time = unsafe {
        let mut sig = [0u8; SIGNATURE_SIZE];
        let mut siglen = 0usize;

        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            dilithium_reference_crypto_sign_signature(
                sig.as_mut_ptr(),
                &mut siglen,
                true_random_seed().as_ptr(),
                SEED_SIZE / 2,
                sk.as_ptr(),
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
