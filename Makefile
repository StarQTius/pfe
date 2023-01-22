all: build

build:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --release

check: rust-dilithium/rsrc/fixtures.txt
	RUSTC_WRAPPER=sccache cd rust-dilithium && cargo test

flash: build
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/rust-dilithium-esp

rust-dilithium/rsrc/fixtures.txt:
	make -C dilithium/ref -j4
	dilithium/ref/test/test_vectors5aes > rust-dilithium/rsrc/fixtures.txt
