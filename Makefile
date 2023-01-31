all: build

build: compare speed speed-keys speed-sign speed-verify speed-verify-fail

speed:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed --release

speed-keys:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed-keys --release

speed-sign:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed-sign --release

speed-verify:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed-verify --release

speed-verify-fail:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example speed-verify-fail --release

compare:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example compare --release

demo-sender:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example demo-sender --release

demo-reader:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --example demo-reader --release

check: rust-dilithium/rsrc/fixtures.txt
	RUSTC_WRAPPER=sccache cd rust-dilithium && cargo test

flash_speed: speed
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed

flash_speed-keys: speed-keys
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed-keys

flash_speed-sign: speed-sign
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed-sign

flash_speed-verify: speed-verify
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed-verify

flash_speed-verify-fail: speed-verify-fail
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/speed-verify-fail

flash_compare: compare
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/compare

flash_demo-sender: demo-sender
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/demo-sender

flash_demo-reader: demo-reader
	espflash rust-dilithium-esp/target/riscv32imc-esp-espidf/release/examples/demo-reader

rust-dilithium/rsrc/fixtures.txt:
	make -C dilithium/ref -j4
	dilithium/ref/test/test_vectors5aes > rust-dilithium/rsrc/fixtures.txt
