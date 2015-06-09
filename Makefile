test:
	cargo test

docs:
	@cargo doc --no-deps

upload-docs: docs
	@./upload-docs.sh

.PHONY: test docs upload-docs
