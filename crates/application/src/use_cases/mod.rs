//! One module per use case.
//!
//! Only `ban_account` is built out. It is the worked example the rest are
//! modelled on: an input struct, an output struct, a struct holding its
//! dependencies as ports, and a single `execute`.

mod ban_account;

pub use ban_account::{BanAccount, BanAccountInput, BanAccountOutput};
