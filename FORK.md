# Keyring adaptations

The JSON span parser accepts unquoted asterisk runs as redacted values and
preserves their original byte ranges, including across HTTP chunks. Quoted
asterisks remain strings. Consumers can inspect disclosed fields and visit
redacted placeholders without assigning them a JSON scalar type.

The HTTP parser checks chunk boundaries and arithmetic, reports malformed
trailers as errors, and handles UTF-8 code points split across chunks.

These parsers describe supplied bytes and spans; they do not authenticate
transcripts or establish the original contents of redacted regions.

CI uses a pinned formatter for consistent formatting.
