PYTHON ?= python3

.PHONY: validate test-runtime

validate:
	$(PYTHON) scripts/validate_schemas.py

test-runtime:
	$(PYTHON) -m unittest discover -s tests/runtime -p 'test_*.py' -t .
