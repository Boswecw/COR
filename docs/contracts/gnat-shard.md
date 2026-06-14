# Gnat Shard

Schema: `schemas/gnat-shard.schema.json`

`GnatShard.v1` is one bounded unit of deterministic syntax extraction work.
It names the run, shard, ordinal, worker type, scoped source reference, media type, source fingerprint, operation, limits, and expected output contract.

GNAT-01 shard worker types are:

- `markdown_syntax`
- `plain_text_syntax`
- `pdf_text_syntax`
- `docx_text_syntax`
- `rtf_text_syntax`
- `odt_text_syntax`
- `epub_text_syntax`

PDF shards are admitted only for bounded text-layer PDF extraction through
`application/pdf`. DOCX shards are admitted only for bounded local DOCX
structure extraction through
`application/vnd.openxmlformats-officedocument.wordprocessingml.document`.
RTF shards are admitted only for bounded paragraph-only local RTF extraction
through `application/rtf` or `text/rtf`. ODT shards are admitted only for
bounded local OpenDocument text extraction through
`application/vnd.oasis.opendocument.text`. EPUB shards are admitted only for
bounded local EPUB package text extraction through `application/epub+zip`.
Other source lanes remain outside the Gnat proving slice until separately
admitted.
