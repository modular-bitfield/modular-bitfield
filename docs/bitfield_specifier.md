Derive macro for Rust `enums` to implement `Specifier` trait.

This allows such an enum to be used as a field of a `#[bitfield]` struct.
The annotated enum must not have any variants with associated data and
by default must have a number of variants that is equal to the power of 2.

If a user wants to circumvent the latter restriction they can add
`#[bits = N]` below the `#[derive(BitfieldSpecifier)]` line in order to
signal to the code generation that the enum may have a relaxed number
of variants.

# Example

## Example: Basic Usage

In the following we define a `MaybeWeekday` enum that lists all weekdays
as well as an invalid day so that we have a power-of-two number of variants.

```
use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier)]
pub enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday, None
}
```

## Example: `#[bits = N]`

If we want to get rid of the `None` variant we need to add `#[bits = 3]`:

```
# use modular_bitfield::prelude::*;
#
#[derive(BitfieldSpecifier)]
#[bits = 3]
pub enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
}
```

## Example: Discriminants

It is possible to explicitly assign discriminants to some of the days.
In our case this is useful since our week starts at sunday:

```
# use modular_bitfield::prelude::*;
#
#[derive(BitfieldSpecifier)]
#[bits = 3]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 0,
}
```

## Example: Use in `#[bitfield]`

Given the above `Weekday` enum that starts at `Sunday` and uses 3 bits in total
we can now use it in a `#[bitfield]` annotated struct as follows:

```
# use modular_bitfield::prelude::*;
#
# #[derive(BitfieldSpecifier)]
# #[bits = 3]
# pub enum Weekday {
#     Monday = 1,
#     Tuesday = 2,
#     Wednesday = 3,
#     Thursday = 4,
#     Friday = 5,
#     Saturday = 6,
#     Sunday = 0,
# }
#[bitfield]
pub struct MeetingTimeSlot {
    day: Weekday,
    from: B6,
    to: B6,
    expired: bool,
}
```

The above `MeetingTimeSlot` uses exactly 16 bits and defines our `Weekday` enum as
compact `day` bitfield. The `from` and `to` require 6 bits each and finally the
`expired` flag requires a single bit.

## Example: Interacting

A user can interact with the above `MeetingTimeSlot` and `Weekday` definitions in
the following ways:

```
# use modular_bitfield::prelude::*;
#
# #[derive(BitfieldSpecifier, Debug, PartialEq)]
# #[bits = 3]
# pub enum Weekday {
#     Monday = 1,
#     Tuesday = 2,
#     Wednesday = 3,
#     Thursday = 4,
#     Friday = 5,
#     Saturday = 6,
#     Sunday = 0,
# }
# #[bitfield]
# pub struct MeetingTimeSlot {
#     day: Weekday,
#     from: B6,
#     to: B6,
#     expired: bool,
# }
#
let mut slot = MeetingTimeSlot::new()
    .with_day(Weekday::Friday)
    .with_from(14) // 14:00 CEST
    .with_to(15); // 15:00 CEST
assert_eq!(slot.day(), Weekday::Friday);
assert_eq!(slot.from(), 14);
assert_eq!(slot.to(), 15);
assert!(!slot.expired());
```
