#include <stdlib.h>

int32_t dilithium_reference_crypto_sign_keypair(uint8_t *pk, uint8_t *sk);
int32_t dilithium_reference_crypto_sign_signature(uint8_t *sig, size_t *siglen, const uint8_t *m, size_t mlen, const uint8_t *sk);
int32_t dilithium_reference_crypto_sign_verify(uint8_t *sig, size_t siglen, const uint8_t *m, size_t mlen, const uint8_t *pk);
