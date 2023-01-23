#![no_std]
#![no_main]

use core::iter::{repeat, zip};
use core::mem::{size_of, MaybeUninit};
use esp_idf_sys::{esp_aes_context, esp_aes_crypt_ecb, esp_aes_init, esp_aes_setkey};
use log::info;
use rust_dilithium::{
    counter::{Counter, SoftwareAesCounter, BLOCK_SIZE, KEY_SIZE},
    make_keys, sign, verify, Signature,
};

struct HardwareAesCounter {
    ctx: esp_aes_context,
    iv: [u8; BLOCK_SIZE],
    counter: u16,
    buf: [u8; BLOCK_SIZE],
    i: usize,
}

impl Counter for HardwareAesCounter {
    fn new(key: &[u8; KEY_SIZE]) -> Self {
        let mut ctx = MaybeUninit::uninit();
        let ctx_ptr = ctx.as_mut_ptr();

        unsafe {
            esp_aes_init(ctx_ptr);
            esp_aes_setkey(ctx_ptr, key.as_ptr(), 256);

            Self {
                ctx: ctx.assume_init(),
                iv: [0u8; BLOCK_SIZE],
                counter: 0,
                buf: [0; BLOCK_SIZE],
                i: BLOCK_SIZE,
            }
        }
    }

    fn reset(&mut self, nonce: u16) {
        self.iv.fill(0);
        self.iv[..2].copy_from_slice(&nonce.to_le_bytes());
        self.counter = 0;
        self.i = BLOCK_SIZE;
    }

    fn squeeze<const N: usize>(&mut self) -> [u8; N] {
        const AES_ENCRYPT: i32 = 1;

        let mut retval = [0; N];
        for x in retval.iter_mut() {
            if self.i == BLOCK_SIZE {
                unsafe {
                    esp_aes_crypt_ecb(
                        &mut self.ctx,
                        AES_ENCRYPT,
                        self.iv.as_ptr(),
                        self.buf.as_mut_ptr(),
                    );
                }

                self.counter += 1;
                self.iv[BLOCK_SIZE - size_of::<u16>()..]
                    .copy_from_slice(&self.counter.to_be_bytes());
                self.i = 0;
            }

            *x = self.buf[self.i];
            self.i += 1;
        }

        retval
    }
}

#[no_mangle]
#[allow(clippy::empty_loop)]
fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let sw_signature = compute_software().unwrap();
    let hw_signature = compute_hardware().unwrap();

    if sw_signature == hw_signature {
        info!("VERIFIED");
    } else {
        info!("ERROR");
        for (sw, hw) in zip(sw_signature.iter(), hw_signature.iter()) {
            if sw == hw {
                info!("{} == {}", sw, hw);
            } else {
                info!("{} != {}", sw, hw);
            }
        }
    }

    loop {}
}

#[inline(never)]
fn compute_software() -> Option<Signature> {
    info!("SOFTWARE");
    info!("KEYS");

    let (pk, sk) = make_keys::<SoftwareAesCounter>(repeat(0)).unwrap();

    info!("SIGN");

    let signature = sign::<SoftwareAesCounter>(&[0u8; 32], &sk);

    info!("VERIFY");

    if verify::<SoftwareAesCounter>(&[0u8; 32], &signature, &pk) {
        Some(signature)
    } else {
        None
    }
}

#[inline(never)]
fn compute_hardware() -> Option<Signature> {
    info!("HARDWARE");
    info!("KEYS");

    let (pk, sk) = make_keys::<HardwareAesCounter>(repeat(0)).unwrap();

    info!("SIGN");

    let signature = sign::<HardwareAesCounter>(&[0u8; 32], &sk);

    info!("VERIFY");

    if verify::<HardwareAesCounter>(&[0u8; 32], &signature, &pk) {
        Some(signature)
    } else {
        None
    }
}
