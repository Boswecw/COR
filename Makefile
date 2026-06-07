PYTHON ?= python3

.PHONY: validate test-runtime test-gnats benchmark-gnats benchmark-gnat-pdf benchmark-gnat-docx test-repo-crawler test-worm

validate:
	$(PYTHON) scripts/validate_schemas.py

test-runtime:
	$(PYTHON) -m unittest discover -s tests/runtime -p 'test_*.py' -t .

test-gnats:
	$(PYTHON) -m unittest discover -s tests/runtime -p 'test_gnat_*.py' -t .

benchmark-gnats:
	$(PYTHON) scripts/benchmark_gnats.py --output docs/benchmarks/gnat-phase4-parallel-proof.md

benchmark-gnat-pdf:
	$(PYTHON) scripts/benchmark_gnat_pdf.py --output docs/benchmarks/gnat-pdf-lane-proof.md

benchmark-gnat-docx:
	$(PYTHON) scripts/benchmark_gnat_docx.py --output docs/benchmarks/gnat-docx-lane-proof.md

test-repo-crawler:
	cargo test --manifest-path repo-crawler/Cargo.toml

test-worm:
	cargo test --manifest-path worm/Cargo.toml
