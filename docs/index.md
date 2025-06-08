Provides macros to support bitfield structs allowing for modular use of bit-enums.

The mainly provided macros are [`#[bitfield]`](bitfield) for structs and
[`#[derive(Specifier)]`](Specifier) for enums that shall be usable
within bitfield structs.

There are preset bitfield specifiers such as `B1`, `B2`,..,`B64`
that allow for easy bitfield usage in structs very similar to how
they work in C or C++.

- Performance of the macro generated code is as fast as its hand-written
  alternative.
- Compile-time checks allow for safe usage of bitfield structs and enums.

### Usage

Annotate a Rust struct with the [`#[bitfield]`](bitfield) attribute in order to convert it into a bitfield,
with [optional parameters](bitfield#parameters) that control how the bitfield is generated.
The `B1`, `B2`, ... `B128` prelude types can be used as primitives to declare the number of bits per field.

```
# use modular_bitfield::prelude::*;
#
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

#### Example: Constructors

```
# use modular_bitfield::prelude::*;
#
# #[bitfield]
# pub struct PackedData {
#     header: B4,
#     body: B9,
#     is_alive: B1,
#     status: B2,
# }
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

#### Example: Primitive Types

Any type that implements the `Specifier` trait can be used as a bitfield field.
Besides the already mentioned `B1`, .. `B128` also the `bool`, `u8`, `u16`, `u32`,
`u64` or `u128` primitive types can be used from prelude.

We can use this knowledge to encode our `is_alive` as `bool` type instead of `B1`:

```
# use modular_bitfield::prelude::*;
#
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

#### Example: Enum Specifiers

It is possible to derive the `Specifier` trait for `enum` types very easily to make
them also usable as a field within a bitfield type:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
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

#### Example: Extra Safety Guard

In order to make sure that our `Status` enum still requires exatly 2 bit we can add
`#[bits = 2]` to its field:

```
# use modular_bitfield::prelude::*;
#
# #[derive(Specifier)]
# pub enum Status {
#     Red, Green, Yellow, None,
# }
#
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

```
# use modular_bitfield::prelude::*;
#
# #[derive(Specifier)]
# #[derive(Debug, PartialEq, Eq)]
# pub enum Status {
#     Red, Green, Yellow, None,
# }
#
# #[bitfield]
# pub struct PackedData {
#     header: B4,
#     body: B9,
#     is_alive: bool,
#     #[bits = 2]
#     status: Status,
# }
#
let mut data = PackedData::new()
    .with_status(Status::Green);
assert_eq!(data.status(), Status::Green);
data.set_status(Status::Red);
assert_eq!(data.status(), Status::Red);
```

#### Example: Skipping Fields

It might make sense to only allow users to set or get information from a field or
even to entirely disallow interaction with a bitfield. For this the `#[skip]` attribute
can be used on a bitfield of a `#[bitfield]` annotated struct.

```
# use modular_bitfield::prelude::*;
#
#[bitfield]
pub struct SomeBitsUndefined {
    #[skip(setters)]
    read_only: bool,
    #[skip(getters)]
    write_only: bool,
    #[skip]
    unused: B6,
}
```

It is possible to use `#[skip(getters, setters)]` or `#[skip(getters)]` followed by a `#[skip(setters)]`
attribute applied on the same bitfield. The effects are the same. When skipping both, getters and setters,
it is possible to completely avoid having to specify a name:

```
# use modular_bitfield::prelude::*;
#
#[bitfield]
pub struct SomeBitsUndefined {
    #[skip] __: B2,
    is_activ: bool,
    #[skip] __: B2,
    is_received: bool,
    #[skip] __: B2,
}
```

#### Example: Unfilled Bitfields

Sometimes it might be useful to not be required to construct a bitfield that defines
all bits and therefore is required to have a bit width divisible by 8. In this case
you can use the `filled: bool` parameter of the `#[bitfield]` macro in order to toggle
this for your respective bitfield:

```
# use modular_bitfield::prelude::*;
#
#[bitfield(filled = false)]
pub struct SomeBitsUndefined {
    is_compact: bool,
    is_secure: bool,
    pre_status: B3,
}
```

In the above example `SomeBitsUndefined` only defines the first 5 bits and leaves the rest
3 bits of its entire 8 bits undefined. The consequences are that its generated `from_bytes`
method is fallible since it must guard against those undefined bits.

#### Example: Recursive Bitfields

It is possible to use `#[bitfield]` structs as fields of `#[bitfield]` structs.
This is generally useful if there are some common fields for multiple bitfields
and is achieved by adding the `#[derive(Specifier)]` attribute to the struct
annotated with `#[bitfield]`:

```
# use modular_bitfield::prelude::*;
#
# #[derive(Specifier)]
# pub enum Status {
#     Red, Green, Yellow, None,
# }
#
#[bitfield(filled = false)]
#[derive(Specifier)]
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
`#[bits: int]` attribute of the `#[derive(Specifier)]` on the `Status` enum we
can have additional compile-time guarantees about the bit widths of the resulting entities:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
#[bits = 2]
pub enum Status {
    Red, Green, Yellow, None,
}

#[bitfield(bits = 4)]
#[derive(Specifier)]
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

#### Example: Advanced Enum Specifiers

For our `Status` enum we actually just need 3 status variants: `Green`, `Yellow` and `Red`.
We introduced the `None` status variants because `Specifier` enums by default are required
to have a number of variants that is a power of two. We can ship around this by specifying
`#[bits = 2]` on the top and get rid of our placeholder `None` variant while maintaining
the invariant of it requiring 2 bits:

```
# use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits = 2]
pub enum Status {
    Red, Green, Yellow,
}
```

However, having such enums now yields the possibility that a bitfield might contain invalid bit
patterns for such fields. We can safely access those fields with protected getters. For the sake
of demonstration we will use the generated `from_bytes` constructor with which we can easily
construct bitfields that may contain invalid bit patterns:

```
# use modular_bitfield::prelude::*;
# use modular_bitfield::error::InvalidBitPattern;
#
# #[derive(Specifier)]
# #[derive(Debug, PartialEq, Eq)]
# #[bits = 2]
# pub enum Status {
#     Red, Green, Yellow,
# }
#
# #[bitfield(filled = false)]
# #[derive(Specifier)]
# pub struct Header {
#     is_compact: bool,
#     is_secure: bool,
#     pre_status: Status,
# }
#
# #[bitfield]
# pub struct PackedData {
#     header: Header,
#     body: B9,
#     is_alive: bool,
#     status: Status,
# }
#
let mut data = PackedData::from_bytes([0b0000_0000, 0b1100_0000]);
//           The 2 status field bits are invalid -----^^
//           as Red = 0x00, Green = 0x01 and Yellow = 0x10
assert_eq!(data.status_or_err(), Err(InvalidBitPattern::new(0b11)));
data.set_status(Status::Green);
assert_eq!(data.status_or_err(), Ok(Status::Green));
```

#### Example: Enum Data Variants

Enums can contain data variants when deriving `Specifier`. All enum bits are used for the actual data,
and you track which variant is active using a separate field:

```
# use modular_bitfield::prelude::*;
#
// Enum with both unit and data variants
#[derive(Specifier, Debug, PartialEq)]
#[bits = 16]  // All 16 bits available for data
enum Event {
    // Unit variants (no data)
    Startup,
    Shutdown,
    // Data variants (each uses all 16 bits for their data)
    Error(u16),    // Error code (0-65535, includes HTTP codes like 404)
    Warning(u16),  // Warning level (0-65535)
}

// Use the enum in a bitfield - track which variant separately
#[bitfield]
pub struct EventLog {
    timestamp: B16,  // 16 bits
    event_type: B4,  // 4 bits - YOU track which variant this is
    #[bits = 16]
    event_data: Event,  // 16 bits - the actual event data
    padding: B4,     // 4 bits - Total: 16 + 4 + 16 + 4 = 40 bits (multiple of 8)
}

// Usage - you decide which variant based on your event_type
let log = EventLog::new()
    .with_timestamp(12345)
    .with_event_type(2)  // You're storing an Error
    .with_event_data(Event::Error(404))  // HTTP error code
    .with_padding(0);

// You handle determining the variant type
match log.event_type() {
    2 => println!("Error occurred"),
    3 => println!("Warning issued"),
    0 => println!("System started"),
    1 => println!("System stopped"),
    _ => println!("Unknown event"),
}
```

Key requirements for enum data variants:
- All data types must implement `Specifier`
- All data variants must have the same bit size as the enum total
- Total enum size must be specified with `#[bits = N]`
- You must track which variant is active in a separate field

#### Example: External Discrimination

Data variants use external discrimination for maximum efficiency:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier, Debug, PartialEq)]
#[bits = 8]  // All 8 bits for data, no internal discrimination
enum Command {
    Read(u8),     // All 8 bits for data
    Write(u8),    // All 8 bits for data
    Delete(u8),   // All 8 bits for data
}

// External discrimination via separate field
#[bitfield]
struct Message {
    command_type: B4,  // External discriminant (0=Read, 1=Write, 2=Delete)
    #[bits = 8]
    command: Command,  // Pure union - no internal discrimination
    padding: B4,
}
```

## Generated Implementations

For the example `#[bitfield]` struct the following implementations are going to be generated:

```
# use modular_bitfield::prelude::*;
#
#[bitfield]
pub struct Example {
    a: bool,
    b: B7,
}
```

| Signature | Description |
|:--|:--|
| `fn new() -> Self` | Creates a new instance of the bitfield with all bits initialized to 0. |
| `fn from_bytes([u8; 1]) -> Self` | Creates a new instance of the bitfield from the given raw bytes. |
| `fn into_bytes(self) -> [u8; 1]` | Returns the underlying bytes of the bitfield. |

And below the generated signatures for field `a`:

| Signature | Description |
|:--|:--|
| `fn a() -> bool` | Returns the value of `a` or panics if invalid. |
| `fn a_or_err() -> Result<bool, InvalidBitPattern<u8>>` | Returns the value of `a` of an error providing information about the invalid bits. |
| `fn set_a(&mut self, new_value: bool)` | Sets `a` to the new value or panics if `new_value` contains invalid bits. |
| `fn set_a_checked(&mut self, new_value: bool) -> Result<(), OutOfBounds>` | Sets `a` to the new value of returns an out of bounds error. |
| `fn with_a(self, new_value: bool) -> Self` | Similar to `set_a` but useful for method chaining. |
| `fn with_a_checked(self, new_value: bool) -> Result<Self, OutOfBounds>` | Similar to `set_a_checked` but useful for method chaining. |

Getters for unnamed fields in tuple-like structs are prefixed with `get_`
(e.g. `get_0()`, `get_1_or_err()`, etc.).

## Generated Structure

From David Tolnay's procedural macro workshop:

The macro conceptualizes given structs as a sequence of bits 0..N.
The bits are grouped into fields in the order specified by the struct written by the user.

The `#[bitfield]` attribute rewrites the caller's struct into a private byte array representation
with public getter and setter methods for each field.
The total number of bits N is required to be a multiple of 8: This is checked at compile time.

### Example

The following invocation builds a struct with a total size of 32 bits or 4 bytes.
It places field `a` in the least significant bit of the first byte,
field `b` in the next three least significant bits,
field `c` in the remaining four most significant bits of the first byte,
and field `d` spanning the next three bytes.

```rust
use modular_bitfield::prelude::*;

#[bitfield]
pub struct MyFourBytes {
    a: B1,
    b: B3,
    c: B4,
    d: B24,
}
```
```text
                               least significant bit of third byte
                                 ┊           most significant
                                 ┊             ┊
                                 ┊             ┊
║  first byte   ║  second byte  ║  third byte   ║  fourth byte  ║
╟───────────────╫───────────────╫───────────────╫───────────────╢
║▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒║
╟─╫─────╫───────╫───────────────────────────────────────────────╢
║a║  b  ║   c   ║                       d                       ║
                 ┊                                             ┊
                 ┊                                             ┊
               least significant bit of d         most significant
```
