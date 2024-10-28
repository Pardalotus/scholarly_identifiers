# Scholarly Identifiers

A library of common identifier types used in Scholarly Publishing metadata. Recognises DOI, ROR, ORCID, and ISBN.

Pre-release, work in progress.

This library is strict about check-digits, but optimistic in recognising input. For example, strings that are formatted as ISBNs with a valid checksum are treated as ISBNs. Strings that appear to be plain DOIs are also recognised.

The representation of identifiers is geared toward stability in representation and comparison. This means that:

 - DOIs are converted to a lower-case format to allow for case-insensitive comparison
 - DOIs are represented without the resolver, allowing for DOIs expressed on different resolvers to be compared
 - Hyphens and spaces are removed from ISBNs so that they can be compared regardless of formatting
 - 10-digit ISBNs are normalised to 13-digit formats, to allow the the same ISBN expressed both way to be compared


# License

This code is MIT Licensed, Copyright 2024 Joe Wass.

Inspired by work done at Crossref by Joe Wass, Dima Safonov, Panos Pandis.
