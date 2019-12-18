//! Main entry point for Delphi

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use delphi::application::APPLICATION;

/// Boot Delphi
fn main() {
    abscissa_core::boot(&APPLICATION);
}
