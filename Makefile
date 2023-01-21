all:
	RUSTC_WRAPPER=sccache cd rust-dilithium-esp && cargo build --release

check: rust-dilithium/rsrc/fixtures.txt
	RUSTC_WRAPPER=sccache cd rust-dilithium && cargo test

rust-dilithium/rsrc/fixtures.txt:
	make -C dilithium/ref -j4
	dilithium/ref/test/test_vectors5aes > rust-dilithium/rsrc/fixtures.txt
