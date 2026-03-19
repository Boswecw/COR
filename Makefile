PYTHON ?= python3

.PHONY: validate

validate:
	$(PYTHON) scripts/validate_schemas.py
