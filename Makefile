.PHONY: format format-check check clippy verify setup-hooks

format:
	cargo fmt --all

format-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

verify: format-check check clippy

setup-hooks:
	chmod +x .githooks/pre-push
	git config core.hooksPath .githooks
	@echo "Git hooks enabled from .githooks"
