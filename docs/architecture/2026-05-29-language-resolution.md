# Language resolution

Date: 2026-05-29

## Decision

`t <lang> <text>` resolves target languages with a bounded whitelist:

- Current ISO 639-1 two-letter language codes are accepted.
- Existing common aliases such as `english`, `japanese`, `日本語`, `jp`, and `zh-TW` remain accepted.
- `kr` remains a compatibility alias for Korean, even though ISO 639-1 assigns `kr` to Kanuri. Kanuri can still be requested with `kanuri`.
- Unknown two-letter strings are not treated as language codes.
- Arbitrary three-or-more-character identifiers are not treated as target languages in positional syntax.

The ISO 639-1 table was derived from the Library of Congress ISO 639-1 RDF/XML list:

https://www.loc.gov/standards/iso639-2/php/xml-iso639-1.php

Deprecated identifiers in that source are excluded from the current-code whitelist.

## Rationale

The CLI intentionally does not maintain a full natural-language capability model. Agent CLIs still decide whether they can translate a requested language well. The local parser only needs enough structure to decide whether the first positional argument is a target language or part of the text.

Accepting every possible two-letter string would misclassify common inputs such as `go home`. Accepting every arbitrary three-or-more-character string would misclassify `t hello world`. A current ISO 639-1 whitelist expands language coverage while keeping this parser boundary explicit.

The `kr` exception preserves existing CLI behavior for users who type Korean by country-style shorthand. Korean remains available by the standard `ko` code and `korean` alias.
