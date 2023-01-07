use std::mem::size_of;

use crypto::{aessafe, symmetriccipher::BlockEncryptor};

const KEY_SIZE: usize = 32;
const BLOCK_SIZE: usize = 16;

pub struct AesCtr {
    encryptor: aessafe::AesSafe256Encryptor,
    iv: [u8; BLOCK_SIZE],
    counter: u16,
    buf: [u8; BLOCK_SIZE],
    i: usize,
}

impl AesCtr {
    pub fn new(key: &[u8; KEY_SIZE], nonce: u16) -> Self {
        let mut iv = [0u8; BLOCK_SIZE];
        iv[..2].copy_from_slice(&nonce.to_le_bytes());

        AesCtr {
            encryptor: aessafe::AesSafe256Encryptor::new(key),
            iv,
            counter: 0,
            buf: [0; BLOCK_SIZE],
            i: BLOCK_SIZE,
        }
    }

    pub fn squeeze<const N: usize>(&mut self) -> [u8; N] {
        let mut retval = [0; N];

        for x in retval.iter_mut() {
            if self.i == BLOCK_SIZE {
                self.encryptor.encrypt_block(&self.iv, &mut self.buf);
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
