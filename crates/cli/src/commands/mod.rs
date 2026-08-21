//! Command handlers.
//!
//! Each handler is thin by design: read the parsed arguments, build a use-case
//! input, call the use case, render the output. Any logic that appears here is
//! logic a web interface would have to duplicate, so it belongs in the
//! application layer instead.

pub(crate) mod accounts;
