use super::{field_config::FieldConfig, BitfieldStruct, Config};

/// Role of a field in variable-size structs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantRole {
    Discriminator,  // Field that determines which variant is active
    Data,          // Field that contains variable-size data
}

/// Compactly stores all shared and useful information about a single `#[bitfield]` field.
pub struct FieldInfo<'a> {
    /// The index of the field.
    pub index: usize,
    /// The actual field.
    pub field: &'a syn::Field,
    /// The configuration of the field.
    pub config: FieldConfig,
    /// The variant role of this field (if any)
    pub variant_role: Option<VariantRole>,
}

impl<'a> FieldInfo<'a> {
    /// Creates a new field info.
    pub fn new(id: usize, field: &'a syn::Field, config: FieldConfig) -> Self {
        let variant_role = if config.is_variant_discriminator() {
            Some(VariantRole::Discriminator)
        } else if config.is_variant_data() {
            Some(VariantRole::Data)
        } else {
            None
        };

        Self {
            index: id,
            field,
            config,
            variant_role,
        }
    }

    /// Returns true if this field is marked as a variant discriminator
    #[allow(dead_code)]
    pub fn is_variant_discriminator(&self) -> bool {
        self.variant_role == Some(VariantRole::Discriminator)
    }

    /// Returns true if this field is marked as variant data
    #[allow(dead_code)]
    pub fn is_variant_data(&self) -> bool {
        self.variant_role == Some(VariantRole::Data)
    }

    /// Returns true if this field is a fixed field (not part of variable bits)
    #[allow(dead_code)]
    pub fn is_fixed_field(&self) -> bool {
        self.variant_role.is_none()
    }

    /// Returns the ident fragment for this field.
    pub fn ident_frag(&self) -> &dyn quote::IdentFragment {
        match &self.field.ident {
            Some(ident) => ident,
            None => &self.index,
        }
    }

    /// Returns the field's identifier as `String`.
    pub fn name(&self) -> String {
        Self::ident_as_string(self.field, self.index)
    }

    /// Returns the field's identifier at the given index as `String`.
    pub fn ident_as_string(field: &'a syn::Field, index: usize) -> String {
        field
            .ident
            .as_ref()
            .map_or_else(|| format!("{index}"), ToString::to_string)
    }
}

impl BitfieldStruct {
    /// Returns an iterator over the names of the fields.
    ///
    /// If a field has no name it is replaced by its field number.
    pub fn fields(item_struct: &syn::ItemStruct) -> impl Iterator<Item = (usize, &syn::Field)> {
        item_struct.fields.iter().enumerate()
    }

    /// Returns an iterator over the names of the fields.
    ///
    /// If a field has no name it is replaced by its field number.
    pub fn field_infos<'a, 'b: 'a>(
        &'a self,
        config: &'b Config,
    ) -> impl Iterator<Item = FieldInfo<'a>> {
        Self::fields(&self.item_struct).map(move |(n, field)| {
            let field_config = config
                .field_configs
                .get(&n)
                .map(|config| &config.value)
                .cloned()
                .unwrap_or_default();
            FieldInfo::new(n, field, field_config)
        })
    }
}
