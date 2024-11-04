//! A library of common identifier types used in Scholarly Publishing metadata. Recognises DOI, ROR, ORCID, and ISBN.
//! Pre-release, work in progress. API subject to change but feedback welcome on the [GitHub repository](https://github.com/Pardalotus/scholarly_identifiers).

mod doi;
pub mod identifiers;
mod isbn;
mod orcid;
mod ror;
mod uri;
