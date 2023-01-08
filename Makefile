all:
	cd rust-dilithium-esp && cargo build

check: rust-dilithium/rsrc/fixtures.txt
	cd rust-dilithium && cargo test

rust-dilithium/rsrc/fixtures.txt:
	make -C dilithium/ref -j4
	dilithium/ref/test/test_vectors5aes > rust-dilithium/rsrc/fixtures.txt
