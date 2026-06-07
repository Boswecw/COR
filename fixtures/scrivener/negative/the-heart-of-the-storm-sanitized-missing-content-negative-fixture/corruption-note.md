# Corruption Note

- started from sanitized derivative
- removed `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/content.rtf`
- retained the `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/` directory with a `synopsis.txt` sidecar so the data directory persists with its expected text body missing
- left readable `*.scrivx` intact
- target BinderItem type: `Text`

The retained directory matters: the degraded condition is a present data directory whose `content.rtf` body is absent, not an entirely absent directory. An empty directory would not survive version control, so a non-body `synopsis.txt` sidecar preserves the directory while keeping `content.rtf` missing.

