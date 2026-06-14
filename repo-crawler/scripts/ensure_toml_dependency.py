#!/usr/bin/env python3
from pathlib import Path
import re
import sys

CARGO_TOML = Path("Cargo.toml")
TARGET_LINE = 'toml = "0.8.23"\n'

def main() -> int:
    if not CARGO_TOML.exists():
        print("Cargo.toml not found in current directory", file=sys.stderr)
        return 1

    text = CARGO_TOML.read_text()

    if re.search(r'(?m)^\s*toml\s*=\s*', text):
        print("toml dependency already present")
        return 0

    match = re.search(r'(?m)^\[dependencies\]\s*$', text)
    if not match:
        print("Could not find [dependencies] section in Cargo.toml", file=sys.stderr)
        return 1

    insert_at = match.end()
    new_text = text[:insert_at] + "\n" + TARGET_LINE + text[insert_at:]

    CARGO_TOML.write_text(new_text)
    print("Inserted toml dependency under [dependencies]")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
