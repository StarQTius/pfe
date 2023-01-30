#include <assert.h>

#include <randombytes.h>
#include <sign.h>
#include <esp_random.h>

#include "bindings.h"

void dilithium_reference_randombytes(uint8_t *out, size_t outlen) {
  esp_fill_random(out, outlen);
}

int32_t dilithium_reference_crypto_sign_keypair(uint8_t *pk, uint8_t *sk) {
  assert(!"`crypto_sign_keypair()` is buggy right now. It is best not to call it before the issue is solved");
  return crypto_sign_keypair(pk, sk);
}

int32_t dilithium_reference_crypto_sign_signature(uint8_t *sig, size_t *siglen, const uint8_t *m, size_t mlen, const uint8_t *sk) {
  return crypto_sign_signature(sig, siglen, m, mlen, sk);
}

int32_t dilithium_reference_crypto_sign_verify(uint8_t *sig, size_t siglen, const uint8_t *m, size_t mlen, const uint8_t *pk) {
  return crypto_sign_verify(sig, siglen, m, mlen, pk);
}
