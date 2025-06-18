#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, rust_2018_idioms)]

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

#[proc_macro_derive(
    Specifier,
    attributes(bits, discriminant, variable_bits, discriminant_bits)
)]
pub fn specifier(input: TokenStream) -> TokenStream {
    bitfield_specifier::generate(input.into()).into()
}

#[deprecated(
    since = "0.12.0",
    note = "use #[derive(Specifier)]. This alias will be removed in 0.13."
)]
#[proc_macro_derive(
    BitfieldSpecifier,
    attributes(bits, discriminant, variable_bits, discriminant_bits)
)]
pub fn bitfield_specifier(input: TokenStream) -> TokenStream {
    bitfield_specifier::generate(input.into()).into()
}

#[cfg(coverage)]
#[test]
fn ui_code_coverage() {
    use runtime_macros::{emulate_attributelike_macro_expansion, emulate_derive_macro_expansion};
    use std::fs::File;

    let mut run_success = true;
    for entry in glob::glob("../tests/ui/**/*.rs").unwrap() {
        let entry = entry.unwrap();
        run_success &= emulate_attributelike_macro_expansion(
            File::open(entry.as_path()).unwrap(),
            &[("bitfield", bitfield::analyse_and_expand)],
        )
        .is_ok();
        run_success &= emulate_derive_macro_expansion(
            File::open(entry.as_path()).unwrap(),
            &[("Specifier", bitfield_specifier::generate)],
        )
        .is_ok();
    }

    assert!(run_success);
}

// Note: The deprecated bitfield_specifier function cannot be tested directly
// because proc macros can only be called from within the proc macro expansion context.
// Its coverage is indirectly tested through the ui tests that use #[derive(BitfieldSpecifier)].
