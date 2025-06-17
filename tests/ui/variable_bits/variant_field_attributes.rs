use modular_bitfield::prelude::*;

// Test: Missing variant_discriminator field
#[bitfield(variable_bits = (32, 64))]
pub struct MissingDiscriminator {
    #[variant_data]
    data: B32,
}

// Test: Missing variant_data field  
#[bitfield(variable_bits = (32, 64))]
pub struct MissingData {
    #[variant_discriminator]
    discriminator: B4,
}

// Test: Multiple variant_discriminator fields
#[bitfield(variable_bits = (32, 64))]
pub struct MultipleDiscriminators {
    #[variant_discriminator]
    disc1: B4,
    #[variant_discriminator]
    disc2: B4,
    #[variant_data]
    data: B24,
}

// Test: Multiple variant_data fields
#[bitfield(variable_bits = (32, 64))]
pub struct MultipleData {
    #[variant_discriminator]
    discriminator: B4,
    #[variant_data]
    data1: B20,
    #[variant_data]
    data2: B8,
}

// Test: Invalid discriminator attribute format
#[bitfield]
pub struct InvalidDiscriminatorFormat {
    #[variant_discriminator = "value"]
    field1: B4,
    field2: B28,
}

// Test: Invalid data attribute format
#[bitfield]
pub struct InvalidDataFormat {
    field1: B4,
    #[variant_data(param)]
    field2: B28,
}

// Test: Using variant attributes without variable_bits
#[bitfield]
pub struct NoVariableBits {
    #[variant_discriminator]
    discriminator: B4,
    #[variant_data]
    data: B28,
}

fn main() {}