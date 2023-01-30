#![no_std]
#![no_main]

use log::info;
use rust_dilithium_esp::{
    compute_hardware, compute_reference, compute_software, true_random_seed, Timer,
};

type Chronometer = Timer<0, 0>;

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    const TRIALS_NB: usize = 10;

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let software_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            compute_software(&true_random_seed(), &true_random_seed());
        });
        chronometer.get()
    };
    info!("Software perf: {software_time} for {TRIALS_NB} iterations");

    let hardware_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            compute_hardware(&true_random_seed(), &true_random_seed());
        });
        chronometer.get()
    };
    info!("Hardware perf: {hardware_time} for {TRIALS_NB} iterations");

    let reference_time = {
        let chronometer = Chronometer::start();

        (0..TRIALS_NB).for_each(|_| {
            compute_reference(&true_random_seed());
        });
        chronometer.get()
    };
    info!("Reference perf: {reference_time} for {TRIALS_NB} iterations");

    loop {}
}
