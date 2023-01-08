use std::mem::size_of;

use aes::{
    cipher::{BlockEncrypt, KeyInit},
    Aes256Enc,
};

pub const KEY_SIZE: usize = 32;
pub const BLOCK_SIZE: usize = 16;

pub trait Counter {
    fn new(key: &[u8; KEY_SIZE]) -> Self;
    fn reset(&mut self, nonce: u16);
    fn squeeze<const N: usize>(&mut self) -> [u8; N];
}

pub struct SoftwareAesCounter {
    encryptor: Aes256Enc,
    iv: [u8; BLOCK_SIZE],
    counter: u16,
    buf: [u8; BLOCK_SIZE],
    i: usize,
}

impl Counter for SoftwareAesCounter {
    fn new(key: &[u8; KEY_SIZE]) -> Self {
        Self {
            encryptor: Aes256Enc::new(From::from(key)),
            iv: [0; BLOCK_SIZE],
            counter: 0,
            buf: [0; BLOCK_SIZE],
            i: BLOCK_SIZE,
        }
    }

    fn reset(&mut self, nonce: u16) {
        self.iv.fill(0);
        self.iv[..2].copy_from_slice(&nonce.to_le_bytes());
        self.counter = 0;
        self.i = BLOCK_SIZE;
    }

    fn squeeze<const N: usize>(&mut self) -> [u8; N] {
        let mut retval = [0; N];

        for x in retval.iter_mut() {
            if self.i == BLOCK_SIZE {
                self.encryptor
                    .encrypt_block_b2b(From::from(&self.iv), From::from(&mut self.buf));
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
