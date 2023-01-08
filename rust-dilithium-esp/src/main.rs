use esp_idf_sys as _;
use rust_dilithium::{counter::SoftwareAesCounter, make_keys};
use std::iter::repeat;

fn main() {
    esp_idf_sys::link_patches();

    println!("START");
    let (pk, _) = make_keys::<SoftwareAesCounter>(repeat(0)).unwrap();

    println!("{pk:?}");
}
