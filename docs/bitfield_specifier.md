Derive macro generating an impl of the trait [`Specifier`].

This macro can be used on all unit enums and structs annotated with
`#[bitfield]`. The enum or struct can be up to 128 bits in size; anything larger
will cause a compilation error.

# Options

* `#[bits = N]`: Explicitly specifies the number of bits used by a unit enum.
  This attribute is required when an enum does not have a power-of-two number of
  variants, but can be used for extra validation no matter what.

# Examples

## Basic usage

In this example, an extra variant (`Invalid`) is required because otherwise the
enum would not contain a power-of-two number of variants. The power-of-two
requirement ensures conversion from raw bits is infallible.

```
use modular_bitfield::prelude::*;

#[derive(Specifier)]
pub enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday, Invalid
}
```

## Using `#[bits = N]`

To eliminate the power-of-two requirement, add the `#[bits]` attribute:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
#[bits = 3]
pub enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
}
```

## Discriminants

Discriminants can be used normally to explicitly override certain values:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
#[bits = 3]
pub enum Weekday {
    Monday = 1,
    Tuesday /* 2 … */,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday = 0,
}
```

## With `#[bitfield]`

An enum that implements `Specifier` can be used normally as a field type in a
`#[bitfield]` struct:

```
# use modular_bitfield::prelude::*;
#
# #[derive(Debug, Eq, PartialEq, Specifier)]
# #[bits = 3]
# pub enum Weekday {
#     Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
# }
#[bitfield]
pub struct MeetingTimeSlot {
    day: Weekday,
    from: B6,
    to: B6,
    expired: bool,
}

let mut slot = MeetingTimeSlot::new()
    .with_day(Weekday::Friday)
    .with_from(14) // 14:00
    .with_to(15); // 15:00
assert_eq!(slot.day(), Weekday::Friday);
assert_eq!(slot.from(), 14);
assert_eq!(slot.to(), 15);
assert!(!slot.expired());
```
