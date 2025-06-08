Derive macro for Rust `enums` to implement `Specifier` trait.

This allows such an enum to be used as a field of a `#[bitfield]` struct.

Enums can have:
- **Unit variants only** (traditional): Simple discriminants without data
- **Data variants** (enhanced): Variants that carry associated data

For unit-only enums, the number of variants must be a power of 2 by default.
To relax this restriction, add `#[bits = N]` to specify the exact bit count.

For enums with data variants, `#[bits = N]` is required to specify the total size.
All data variants must have the same bit size.

# Example

## Example: Basic Usage

In the following we define a `MaybeWeekday` enum that lists all weekdays
as well as an invalid day so that we have a power-of-two number of variants.

```
use modular_bitfield::prelude::*;

#[derive(Specifier)]
pub enum Weekday {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday, None
}
```

## Example: `#[bits = N]`

If we want to get rid of the `None` variant we need to add `#[bits = 3]`:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
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
#[derive(Specifier)]
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
# #[derive(Specifier)]
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
# #[derive(Specifier, Debug, PartialEq)]
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

## Example: Enum Data Variants

Enums can contain variants with associated data. When used as bitfield specifiers,
all enum bits are used for the data - you track which variant is active separately:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier, Debug, PartialEq)]
#[bits = 8]  // All 8 bits used for data
enum Event {
    // Unit variants (no data)
    Startup,
    Shutdown,
    // Data variants (each uses all 8 bits for their data)
    Error(u8),    // Error code uses all 8 bits
    Warning(u8),  // Warning level uses all 8 bits
}

#[bitfield]
pub struct SystemLog {
    timestamp: B16,
    event_type: u8,  // You track which variant is active (0=Startup, 1=Shutdown, 2=Error, 3=Warning)
    #[bits = 8]
    event_data: Event, // The actual data - all 8 bits available
    source: B8,
}

// Usage - you decide which variant to use based on your event_type
let log = SystemLog::new()
    .with_timestamp(1234)
    .with_event_type(2)  // You're storing an Error
    .with_event_data(Event::Error(42))  // Error code 42
    .with_source(1);

// You handle determining the variant type
match log.event_type() {
    2 => println!("Error occurred"),
    3 => println!("Warning issued"),
    _ => println!("System event"),
}
```

## Example: Mixed Unit and Data Variants

You can mix unit variants (no data) with data variants in the same enum:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
#[bits = 8]  // All 8 bits available for data when needed
enum Message {
    Empty,                    // Unit variant (no data)
    PriorityMsg(u8),         // Data variant (uses all 8 bits for priority)
    StatusMsg(u8),           // Data variant (uses all 8 bits for status)  
    Reset,                   // Unit variant (no data)
}

// Usage in bitfields - you track which variant is active
#[bitfield]
struct Packet {
    timestamp: B10,
    message_type: B4,        // You track which Message variant this is
    #[bits = 8]
    message_data: Message,   // The message data
    padding: B2,
}
```

### Requirements for Data Variants

1. **All data types must implement `Specifier`**
2. **All data variants must have the same `BITS` value as the enum total**
3. **Total enum size must be specified with `#[bits = N]`**
4. **Unit variants can be mixed with data variants**
5. **You must track which variant is active separately** - the enum doesn't store this information

When using data variants, all enum bits are used for the actual data.
You need to track which variant is active using a separate field:

```
# use modular_bitfield::prelude::*;
#
#[derive(Specifier)]
#[bits = 8]  // All 8 bits available for data
enum Command {
    Noop,             // Unit variant (no data)
    Read(u8),         // Data variant - all 8 bits for read parameter
    Write(u8),        // Data variant - all 8 bits for write parameter  
    Delete(u8),       // Data variant - all 8 bits for delete parameter
}

// Track which variant is active separately
struct Message {
    command_type: u8, // You store which variant this is (0=Noop, 1=Read, 2=Write, 3=Delete)
    command_data: Command, // The actual command data
}
```

### Bit Layout

Data variant enums use all their bits for the actual data:
```text
[all_bits_for_data]
```

Example with 8-bit enum:
- `Command::Noop` → `0x00` (unit variant, no data)
- `Command::Read(42)` → `0x2A` (u8 value 42, uses all 8 bits)
- `Command::Write(255)` → `0xFF` (u8 value 255, uses all 8 bits)

You track which variant is active using a separate field in your bitfield.
