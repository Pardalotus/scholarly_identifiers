# Scholarly Identifiers

A library of common identifier types used in Scholarly Publishing metadata.
Recognises DOI, ROR, ORCID, and ISBN. More ocming.

Pre-release, work in progress. API subject to change but feedback welcome on the
[GitHub repository](https://github.com/Pardalotus/scholarly_identifiers).

This library is strict about check-digits, but optimistic in recognising input.
For example, strings that are formatted as ISBNs with a valid checksum are
treated as ISBNs. Strings that appear to be plain DOIs are also recognised.

The representation of identifiers is geared toward stability in representation
and comparison. This means that:

 - DOIs are converted to a lower-case format to allow for case-insensitive comparison
 - DOIs are represented without the resolver, allowing for DOIs expressed on different resolvers to be compared
 - Hyphens and spaces are removed from ISBNs so that they can be compared regardless of formatting
 - 10-digit ISBNs are normalised to 13-digit formats, to allow the the same ISBN expressed both way to be compared

# Why use this library?

If you're using scholarly metadata, you'll likely be using scholarly
identifiers. Each identifier has its own rules for parsing, validation, and
representation. More info on the [Pardalotus Blog](https://pardalotus.tech/posts).

This library will help with that.

Features:
 - Recognises DOI, ISBN, ORCID, ROR. More coming.
 - Validation for those types that have checksums.
 - Normalisation, according to each type's rules.
 - URI representation, where appropriate for each type.
 - Stable string representation and type IDs, for use in database keys.

# Try it out

See the examples:

```
cargo run --example main
```

# License

This code is MIT Licensed, Copyright 2024 Joe Wass.

The code is inspired by work done at Crossref by Joe Wass, Dima Safonov, Panos Pandis.
