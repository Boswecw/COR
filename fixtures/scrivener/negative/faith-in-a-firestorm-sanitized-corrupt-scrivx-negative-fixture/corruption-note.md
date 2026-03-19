# Corruption Note

The authoritative `*.scrivx` in this packet was intentionally corrupted in a sanitized copy.

Corruption performed:
- removed the closing `</ScrivenerProject>` tag
- injected a malformed fragment immediately after `<Binder>`

The goal is to preserve an otherwise package-shaped `.scriv` directory while making authority parsing fail closed.
