use super::{field_config::FieldConfig, BitfieldStruct, Config};

/// Compactly stores all shared and useful information about a single `#[bitfield]` field.
pub struct FieldInfo<'a> {
    /// The index of the field.
    pub index: usize,
    /// The actual field.
    pub field: &'a syn::Field,
    /// The configuration of the field.
    pub config: FieldConfig,
}

impl<'a> FieldInfo<'a> {
    /// Creates a new field info.
    pub fn new(id: usize, field: &'a syn::Field, config: FieldConfig) -> Self {
        Self {
            index: id,
            field,
            config,
        }
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
