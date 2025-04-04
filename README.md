# Modular Bitfields for Rust

[![crates](https://img.shields.io/crates/v/modular-bitfield.svg)](https://crates.io/crates/modular-bitfield)
[![tests](https://github.com/Robbepop/modular-bitfield/actions/workflows/rust.yml/badge.svg)](https://github.com/Robbepop/modular-bitfield/actions/workflows/rust.yml)
[![docs.rs](https://docs.rs/modular-bitfield/badge.svg)](https://docs.rs/modular-bitfield)
[![codecov](https://codecov.io/gh/Robbepop/modular-bitfield/branch/master/graph/badge.svg)](https://codecov.io/gh/Robbepop/modular-bitfield)

- `no_std`: Supports embedded development without `std` library.
- This crate uses and generates 100% safe Rust code.

## Description

Allows to have bitfield structs and enums as bitfield specifiers that work very similar to C and C++ bitfields.

## Advantages

- **Safety:** Macro embraced enums and structs are checked for valid structure during compilation time.
- **Speed:** Generated code is as fast as handwritten code. (See benchmarks below.)
- **Modularity:** Enums can be used modular within bitfield structs.

## Attribution

Implements the `#[bitfield]` macros introduced and specified in David Tolnay's [procedural macro workshop][procedural-macro-workshop].

Thanks go to David Tolnay for designing the specification for the macros implemented in this crate.

## Usage

Annotate a Rust struct with the `#[bitfield]` attribute in order to convert it into a bitfield.
The `B1`, `B2`, ... `B128` prelude types can be used as primitives to declare the number of bits per field.

```rust
#[bitfield]
pub struct PackedData {
    header: B4,
    body: B9,
    is_alive: B1,
    status: B2,
}
 ```

This produces a `new` constructor as well as a variety of getters and setters that
allows to interact with the bitfield in a safe fashion:

### Example: Constructors

```rust
let data = PackedData::new()
    .with_header(1)
    .with_body(2)
    .with_is_alive(0)
    .with_status(3);
assert_eq!(data.header(), 1);
assert_eq!(data.body(), 2);
assert_eq!(data.is_alive(), 0);
assert_eq!(data.status(), 3);
```

### Example: Primitive Types

Any type that implements the `Specifier` trait can be used as a bitfield field.
Besides the already mentioned `B1`, .. `B128` also the `bool`, `u8`, `u16`, `u32`,
`u64` or `u128` primitive types can be used from prelude.

We can use this knowledge to encode our `is_alive` as `bool` type instead of `B1`:

```rust
#[bitfield]
pub struct PackedData {
    header: B4,
    body: B9,
    is_alive: bool,
    status: B2,
}

let mut data = PackedData::new()
    .with_is_alive(true);
assert!(data.is_alive());
data.set_is_alive(false);
assert!(!data.is_alive());
```

### Example: Enum Specifiers

It is possible to derive the `Specifier` trait for `enum` types very easily to make
them also usable as a field within a bitfield type:

```rust
#[derive(BitfieldSpecifier)]
pub enum Status {
    Red, Green, Yellow, None,
}

#[bitfield]
pub struct PackedData {
    header: B4,
    body: B9,
    is_alive: bool,
    status: Status,
}
```

### Example: Extra Safety Guard

In order to make sure that our `Status` enum still requires exatly 2 bit we can add
`#[bits = 2]` to its field:

```rust
#[bitfield]
pub struct PackedData {
    header: B4,
    body: B9,
    is_alive: bool,
    #[bits = 2]
    status: Status,
}
```

Setting and getting our new `status` field is naturally as follows:

```rust
let mut data = PackedData::new()
    .with_status(Status::Green);
assert_eq!(data.status(), Status::Green);
data.set_status(Status::Red);
assert_eq!(data.status(), Status::Red);
```

### Example: Recursive Bitfields

It is possible to use `#[bitfield]` structs as fields of `#[bitfield]` structs.
This is generally useful if there are some common fields for multiple bitfields
and is achieved by adding `#[derive(BitfieldSpecifier)]` to the attributes of the
`#[bitfield]` annotated struct:

```rust
#[bitfield]
#[derive(BitfieldSpecifier)]
pub struct Header {
    is_compact: bool,
    is_secure: bool,
    pre_status: Status,
}

#[bitfield]
pub struct PackedData {
    header: Header,
    body: B9,
    is_alive: bool,
    status: Status,
}
```

With the `bits: int` parameter of the `#[bitfield]` macro on the `Header` struct and the
`#[bits: int]` attribute of the `#[derive(BitfieldSpecifier)]` on the `Status` enum we
can have additional compile-time guarantees about the bit widths of the resulting entities:

```rust
#[derive(BitfieldSpecifier)]
#[bits = 2]
pub enum Status {
    Red, Green, Yellow
}

#[bitfield(bits = 4)]
#[derive(BitfieldSpecifier)]
pub struct Header {
    is_compact: bool,
    is_secure: bool,
    #[bits = 2]
    pre_status: Status,
}

#[bitfield(bits = 16)]
pub struct PackedData {
    #[bits = 4]
    header: Header,
    body: B9,
    is_alive: bool,
    #[bits = 2]
    status: Status,
}
```

### Example: Advanced Enum Specifiers

For our `Status` enum we actually just need 3 status variants: `Green`, `Yellow` and `Red`.
We introduced the `None` status variants because `Specifier` enums by default are required
to have a number of variants that is a power of two. We can ship around this by specifying
`#[bits = 2]` on the top and get rid of our placeholder `None` variant while maintaining
the invariant of it requiring 2 bits:

```rust
# use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier)]
#[bits = 2]
pub enum Status {
    Red, Green, Yellow,
}
```

However, having such enums now yields the possibility that a bitfield might contain invalid bit
patterns for such fields. We can safely access those fields with protected getters. For the sake
of demonstration we will use the generated `from_bytes` constructor with which we can easily
construct bitfields that may contain invalid bit patterns:

```rust
let mut data = PackedData::from_bytes([0b0000_0000, 0b1100_0000]);
//           The 2 status field bits are invalid -----^^
//           as Red = 0x00, Green = 0x01 and Yellow = 0x10
assert_eq!(data.status_or_err(), Err(InvalidBitPattern { invalid_bytes: 0b11 }));
data.set_status(Status::Green);
assert_eq!(data.status_or_err(), Ok(Status::Green));
```

## Benchmarks

Below are some benchmarks between the [hand-written code][benchmark-code] and the macro-generated code for some example getters and setters that cover a decent variety of use cases.

We can conclude that the macro-generated code is as fast as hand-written code would be. Please file a PR if you see a way to improve either side.

- `cargo bench` to run the benchmarks
- `cargo test --benches` to run the benchmark tests

[Click here to view all benchmark results.][benchmark-results]

[benchmark-code]: https://github.com/Robbepop/modular-bitfield/blob/master/benches/handwritten.rs
[benchmark-results]: https://gist.github.com/Robbepop/bcff4fe149e0e622b752f0eb07b31880

### Summary

The `modular_bitfield` crate generates bitfields that are ...

- just as efficient as the handwritten alternatives.
- equally efficient or more efficient than the alternative [bitfield] crate.

[bitfield]: https://crates.io/crates/bitfield

### Showcase: Generated vs Handwritten

We tested the following `#[bitfield]` `struct`:

```rust
#[bitfield]
pub struct Generated {
    pub a: B9,  // Spans 2 bytes.
    pub b: B6,  // Within 2nd byte.
    pub c: B13, // Spans 3 bytes.
    pub d: B1,  // Within 4rd byte.
    pub e: B3,  // Within 4rd byte.
    pub f: B32, // Spans rest 4 bytes.
}
```

**Note:** All benchmarks timing results sum 10 runs each.

### Getter Performance

```
get_a/generated     time:   [3.0990 ns 3.1119 ns 3.1263 ns]
get_a/handwritten   time:   [3.1072 ns 3.1189 ns 3.1318 ns]

get_b/generated     time:   [3.0859 ns 3.0993 ns 3.1140 ns]
get_b/handwritten   time:   [3.1062 ns 3.1154 ns 3.1244 ns]

get_c/generated     time:   [3.0892 ns 3.1140 ns 3.1491 ns]
get_c/handwritten   time:   [3.1031 ns 3.1144 ns 3.1266 ns]

get_d/generated     time:   [3.0937 ns 3.1055 ns 3.1182 ns]
get_d/handwritten   time:   [3.1109 ns 3.1258 ns 3.1422 ns]

get_e/generated     time:   [3.1009 ns 3.1139 ns 3.1293 ns]
get_e/handwritten   time:   [3.1217 ns 3.1366 ns 3.1534 ns]

get_f/generated     time:   [3.1064 ns 3.1164 ns 3.1269 ns]
get_f/handwritten   time:   [3.1067 ns 3.1221 ns 3.1404 ns]
```

### Setter Performance

```
set_a/generated     time:   [15.784 ns 15.855 ns 15.932 ns]
set_a/handwritten   time:   [15.841 ns 15.907 ns 15.980 ns]

set_b/generated     time:   [20.496 ns 20.567 ns 20.643 ns]
set_b/handwritten   time:   [20.319 ns 20.384 ns 20.454 ns]

set_c/generated     time:   [19.155 ns 19.362 ns 19.592 ns]
set_c/handwritten   time:   [19.265 ns 19.383 ns 19.523 ns]

set_d/generated     time:   [12.325 ns 12.376 ns 12.429 ns]
set_d/handwritten   time:   [12.416 ns 12.472 ns 12.541 ns]

set_e/generated     time:   [20.460 ns 20.528 ns 20.601 ns]
set_e/handwritten   time:   [20.473 ns 20.534 ns 20.601 ns]

set_f/generated     time:   [6.1466 ns 6.1769 ns 6.2127 ns]
set_f/handwritten   time:   [6.1467 ns 6.1962 ns 6.2670 ns]
```

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this codebase by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

[procedural-macro-workshop]: https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md
