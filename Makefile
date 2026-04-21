PYTHON ?= python3

.PHONY: validate test-runtime test-repo-crawler

validate:
	$(PYTHON) scripts/validate_schemas.py

test-runtime:
	$(PYTHON) -m unittest discover -s tests/runtime -p 'test_*.py' -t .

test-repo-crawler:
	cargo test --manifest-path repo-crawler/Cargo.toml
