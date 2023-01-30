set(DILITHIUM_REF_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../dilithium/ref)

# We build the CRYSTALS-Dilithium reference implementation as an ESP component
idf_component_register(SRCS
  "${DILITHIUM_REF_PATH}/sign.c"
	"${DILITHIUM_REF_PATH}/packing.c"
	"${DILITHIUM_REF_PATH}/polyvec.c"
	"${DILITHIUM_REF_PATH}/poly.c"
	"${DILITHIUM_REF_PATH}/ntt.c"
	"${DILITHIUM_REF_PATH}/reduce.c"
	"${DILITHIUM_REF_PATH}/rounding.c"
	"${DILITHIUM_REF_PATH}/fips202.c"
	"${DILITHIUM_REF_PATH}/aes256ctr.c"
	"${DILITHIUM_REF_PATH}/symmetric-aes.c"
  "bindings.c"
  INCLUDE_DIRS ${DILITHIUM_REF_PATH})

# We are using CRYSTALS-Dilithium AES 5
# Due to name clash with some components of the Espressif standard library,
# `randombytes` is renamed `dilithium_reference_randombytes` through macro replacement
target_compile_definitions(${COMPONENT_LIB} PRIVATE
  DILITHIUM_USE_AES
  DILITHIUM_MODE=5
  randombytes=dilithium_reference_randombytes)
