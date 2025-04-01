#![recursion_limit = "256"]
#![forbid(unsafe_code)]

extern crate proc_macro;

#[macro_use]
mod errors;
mod bitfield;
mod bitfield_specifier;
mod define_specifiers;

use proc_macro::TokenStream;

/// Generates the `B1`, `B2`, ..., `B128` bitfield specifiers.
///
/// Only of use witihn the `modular_bitfield` crate itself.
#[proc_macro]
pub fn define_specifiers(input: TokenStream) -> TokenStream {
    define_specifiers::generate(input.into()).into()
}

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    bitfield::analyse_and_expand(args.into(), input.into()).into()
}

#[proc_macro_derive(BitfieldSpecifier, attributes(bits))]
pub fn bitfield_specifier(input: TokenStream) -> TokenStream {
    bitfield_specifier::generate(input.into()).into()
}

#[cfg(coverage)]
#[test]
fn ui_code_coverage() {
    use runtime_macros::{emulate_attributelike_macro_expansion, emulate_derive_macro_expansion};
    use std::fs::File;

    let mut run_success = true;
    for entry in glob::glob("../tests/ui/*/**/*.rs").unwrap() {
        let entry = entry.unwrap();
        run_success &= emulate_attributelike_macro_expansion(
            File::open(entry.as_path()).unwrap(),
            &[("bitfield", bitfield::analyse_and_expand)],
        )
        .is_ok();
        run_success &= emulate_derive_macro_expansion(
            File::open(entry.as_path()).unwrap(),
            &[("BitfieldSpecifier", bitfield_specifier::generate)],
        )
        .is_ok();
    }

    assert!(run_success);
}
