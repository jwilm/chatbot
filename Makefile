.PHONY: test
test:
	cargo test --features 'slack-adapter irc-adapter'

.PHONY: docs
docs:
	cargo doc --features 'slack-adapter irc-adapter' --no-deps
