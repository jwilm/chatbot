test:
	cargo test --features 'slack-adapter irc-adapter'

docs:
	@cargo doc --no-deps

upload-docs: docs
	@./upload-docs.sh

.PHONY: test docs upload-docs
