#![no_std]

use core::mem::{size_of, transmute, MaybeUninit};
use esp_idf_sys::{
    dilithium_reference_crypto_sign_keypair, dilithium_reference_crypto_sign_signature,
    dilithium_reference_crypto_sign_verify, esp_aes_context, esp_aes_crypt_ecb, esp_aes_init,
    esp_aes_setkey, esp_fill_random, timer_alarm_t_TIMER_ALARM_DIS,
    timer_autoreload_t_TIMER_AUTORELOAD_DIS, timer_config_t, timer_count_dir_t_TIMER_COUNT_UP,
    timer_deinit, timer_get_counter_value, timer_init, timer_intr_t_TIMER_INTR_NONE, timer_pause,
    timer_set_counter_value, timer_src_clk_t_TIMER_SRC_CLK_APB, timer_start,
    timer_start_t_TIMER_PAUSE,
};
use rust_dilithium::{
    counter::{Counter, SoftwareAesCounter, BLOCK_SIZE, KEY_SIZE},
    make_keys, sign, verify, Seed, Signature, PUBLIC_KEY_SIZE, SECRET_KEY_SIZE, SIGNATURE_SIZE,
};

pub struct HardwareAesCounter {
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

pub struct Timer<const GROUP: u32, const TIMER: u32>;

impl<const GROUP: u32, const TIMER: u32> Timer<GROUP, TIMER> {
    pub fn start() -> Self {
        unsafe {
            let config = timer_config_t {
                clk_src: timer_src_clk_t_TIMER_SRC_CLK_APB,
                alarm_en: timer_alarm_t_TIMER_ALARM_DIS,
                counter_en: timer_start_t_TIMER_PAUSE,
                intr_type: timer_intr_t_TIMER_INTR_NONE,
                counter_dir: timer_count_dir_t_TIMER_COUNT_UP,
                auto_reload: timer_autoreload_t_TIMER_AUTORELOAD_DIS,
                divider: 2,
            };

            timer_init(GROUP, TIMER, &config);
            timer_set_counter_value(GROUP, TIMER, 0);
            timer_start(GROUP, TIMER);
        }

        Self {}
    }

    pub fn pause(&self, f: impl FnOnce()) {
        unsafe {
            timer_pause(GROUP, TIMER);
            f();
            timer_start(GROUP, TIMER);
        }
    }

    pub fn get(&self) -> u64 {
        let mut retval = 0u64;

        unsafe {
            timer_get_counter_value(GROUP, TIMER, &mut retval as *mut u64);
        }

        retval
    }
}

impl<const GROUP: u32, const TIMER: u32> Drop for Timer<GROUP, TIMER> {
    fn drop(&mut self) {
        unsafe {
            timer_deinit(GROUP, TIMER);
        }
    }
}

#[inline(never)]
pub fn compute_software(msg: &[u8], seed: &Seed) -> Option<Signature> {
    let (pk, sk) = make_keys::<SoftwareAesCounter>(seed).unwrap();
    let signature = sign::<SoftwareAesCounter>(msg, &sk);
    if verify::<SoftwareAesCounter>(msg, &signature, &pk) {
        Some(signature)
    } else {
        None
    }
}

#[inline(never)]
pub fn compute_hardware(msg: &[u8], seed: &Seed) -> Option<Signature> {
    let (pk, sk) = make_keys::<HardwareAesCounter>(seed).unwrap();
    let signature = sign::<HardwareAesCounter>(msg, &sk);
    if verify::<HardwareAesCounter>(msg, &signature, &pk) {
        Some(signature)
    } else {
        None
    }
}

#[inline(never)]
pub fn compute_reference(msg: &[u8]) -> Option<Signature> {
    // The compiler might not ellide these variables initialization, which would cause an overhead.
    // However, it is negligible compared to the execution time of the algorithm.
    let mut pk = [0u8; PUBLIC_KEY_SIZE];
    let mut sk = [0u8; SECRET_KEY_SIZE];
    let mut sig = [0u8; SIGNATURE_SIZE];
    let mut siglen = 0usize;

    unsafe {
        dilithium_reference_crypto_sign_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
        dilithium_reference_crypto_sign_signature(
            sig.as_mut_ptr(),
            &mut siglen,
            msg.as_ptr(),
            msg.len(),
            sk.as_ptr(),
        );
        if dilithium_reference_crypto_sign_verify(
            sig.as_mut_ptr(),
            siglen,
            msg.as_ptr(),
            msg.len(),
            pk.as_mut_ptr(),
        ) == 0
        {
            Some(sig)
        } else {
            None
        }
    }
}

pub fn true_random_seed() -> Seed {
    let mut retval = Seed::default();

    unsafe {
        esp_fill_random(transmute(retval.as_mut_ptr()), retval.len());
    }

    retval
}
