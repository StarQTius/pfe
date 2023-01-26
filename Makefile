all: build

build: compare speed

speed:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed --release

compare:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example compare --release

check: rust-dilithium/rsrc/fixtures.txt
	RUSTC_WRAPPER=sccache cd rust-dilithium && cargo test

flash_speed: speed
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed

flash_compare: compare
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/compare

rust-dilithium/rsrc/fixtures.txt:
	make -C dilithium/ref -j4
	dilithium/ref/test/test_vectors5aes > rust-dilithium/rsrc/fixtures.txt
