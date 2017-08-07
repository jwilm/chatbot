test:
	cargo test --features 'slack-adapter irc-adapter'

docs:
	@cargo doc --no-deps

.PHONY: test docs
