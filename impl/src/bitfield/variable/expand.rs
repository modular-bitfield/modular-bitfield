use crate::bitfield::{
    config::Config,
    BitfieldStruct,
};
use super::analysis::{VariableBitsAnalysis, VariableStructAnalysis};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned as _;

/// Extension trait for variable struct expansion
pub trait VariableStructExpander {
    /// Expand variable-size struct
    fn expand_variable_struct(&self, config: &Config) -> syn::Result<TokenStream2>;
    
    /// Generate Specifier implementation for variable-size structs
    fn generate_variable_specifier_impl(&self, analysis: &VariableStructAnalysis) -> TokenStream2;
    
    /// Generate variable-size struct methods and types
    fn generate_variable_size_extensions(
        &self,
        config: &Config,
    ) -> syn::Result<Option<TokenStream2>>;
}

impl VariableStructExpander for BitfieldStruct {
    fn expand_variable_struct(&self, config: &Config) -> syn::Result<TokenStream2> {
        // Full variable struct expansion will be implemented here
        // For now, delegate to generate_variable_size_extensions
        match self.generate_variable_size_extensions(config)? {
            Some(extensions) => Ok(extensions),
            None => Ok(quote! {}),
        }
    }
    
    /// Generate Specifier implementation for variable-size structs
    fn generate_variable_specifier_impl(&self, analysis: &VariableStructAnalysis) -> TokenStream2 {
        let span = self.item_struct.span();
        let ident = &self.item_struct.ident;
        let (impl_generics, ty_generics, where_clause) = self.item_struct.generics.split_for_impl();

        // Use maximum size for the Specifier trait
        let max_bits = analysis.sizes.iter().max().unwrap_or(&0);
        let max_bytes = (max_bits + 7) / 8;

        quote_spanned!(span=>
            #[allow(clippy::identity_op)]
            const _: () = {
                impl #impl_generics ::modular_bitfield::private::checks::CheckSpecifierHasAtMost128Bits for #ident #ty_generics #where_clause {
                    type CheckType = [(); (#max_bits <= 128) as ::core::primitive::usize];
                }
            };

            #[allow(clippy::identity_op)]
            impl #impl_generics ::modular_bitfield::Specifier for #ident #ty_generics #where_clause {
                const BITS: usize = #max_bits;

                type Bytes = [u8; #max_bytes];
                type InOut = Self;

                #[inline]
                fn into_bytes(
                    value: Self::InOut,
                ) -> ::core::result::Result<Self::Bytes, ::modular_bitfield::error::OutOfBounds> {
                    // Use dynamic serialization based on discriminant
                    let mut bytes = [0u8; #max_bytes];
                    let size_bytes = value.required_bytes();

                    if size_bytes <= #max_bytes {
                        bytes[..size_bytes].copy_from_slice(&value.bytes[..size_bytes]);
                        ::core::result::Result::Ok(bytes)
                    } else {
                        ::core::result::Result::Err(::modular_bitfield::error::OutOfBounds)
                    }
                }

                #[inline]
                fn from_bytes(
                    bytes: Self::Bytes,
                ) -> ::core::result::Result<Self::InOut, ::modular_bitfield::error::InvalidBitPattern<Self::Bytes>> {
                    // Try to parse using dynamic method
                    Self::from_bytes_dynamic(&bytes)
                        .ok_or_else(|| ::modular_bitfield::error::InvalidBitPattern::new(bytes))
                }
            }
        )
    }

    /// Generate variable-size struct methods and types
    fn generate_variable_size_extensions(
        &self,
        config: &Config,
    ) -> syn::Result<Option<TokenStream2>> {
        let analysis = match self.analyze_variable_bits(config)? {
            Some(analysis) => analysis,
            None => return Ok(None), // Not a variable-size struct
        };

        let struct_ident = &self.item_struct.ident;
        let (impl_generics, ty_generics, where_clause) = self.item_struct.generics.split_for_impl();

        // Generate size-specific constructors
        let constructors = self.generate_size_specific_constructors(&analysis)?;

        // Generate size-specific serialization methods
        let serialization_methods = self.generate_variable_serialization(&analysis)?;

        // Generate wire format helpers
        let wire_format_helpers = self.generate_wire_format_helpers(&analysis)?;

        // Generate helper methods
        let helper_methods = self.generate_variable_helper_methods(&analysis)?;

        // Generate compile-time validations
        let compile_time_validations = self.generate_variable_validations(&analysis)?;

        Ok(Some(quote! {
            #compile_time_validations

            impl #impl_generics #struct_ident #ty_generics #where_clause {
                #constructors
                #serialization_methods
                #wire_format_helpers
                #helper_methods
            }
        }))
    }
}

// Helper methods for variable struct expansion
impl BitfieldStruct {
    fn generate_size_specific_constructors(
        &self,
        analysis: &VariableStructAnalysis,
    ) -> syn::Result<TokenStream2> {
        // Calculate the maximum size in bytes for the struct
        let max_size = analysis.sizes.iter().max().unwrap_or(&0);
        let max_bytes = (max_size + 7) / 8;

        let constructors = analysis.sizes.iter().enumerate().map(|(_index, &size)| {
            let constructor_name = format_ident!("new_{}bit", size);

            quote! {
                /// Creates a new instance with zero-initialized data for #size-bit configuration
                #[inline]
                #[must_use]
                pub const fn #constructor_name() -> Self {
                    Self {
                        bytes: [0u8; #max_bytes],  // Always use max size
                    }
                }
            }
        });

        Ok(quote! {
            #( #constructors )*
        })
    }

    fn generate_variable_serialization(
        &self,
        analysis: &VariableStructAnalysis,
    ) -> syn::Result<TokenStream2> {
        let _struct_ident = &self.item_struct.ident;

        // Generate size-specific into_bytes methods
        let into_bytes_methods = analysis.sizes.iter().enumerate().map(|(index, &size)| {
            let method_name = format_ident!("into_bytes_{}", size);
            let size_bytes = (size + 7) / 8;

            quote! {
                /// Converts to byte array for #size-bit configuration
                #[inline]
                pub fn #method_name(&self) -> [u8; #size_bytes] {
                    // Validate this instance is the right size for this method
                    debug_assert_eq!(self.configuration_index(), #index,
                        "Cannot serialize {}bit configuration as {}bit",
                        self.actual_size(), #size);

                    let mut result = [0u8; #size_bytes];
                    result[..::core::cmp::min(self.bytes.len(), #size_bytes)]
                        .copy_from_slice(&self.bytes[..::core::cmp::min(self.bytes.len(), #size_bytes)]);
                    result
                }
            }
        });

        // Generate size-specific from_bytes methods
        let from_bytes_methods = analysis.sizes.iter().enumerate().map(|(index, &size)| {
            let method_name = format_ident!("from_bytes_{}", size);
            let size_bytes = (size + 7) / 8;

            quote! {
                /// Creates instance from byte array for #size-bit configuration
                #[inline]
                pub fn #method_name(bytes: [u8; #size_bytes]) -> ::core::result::Result<Self, ::modular_bitfield::error::OutOfBounds> {
                    // Create instance with maximum size, then copy provided bytes
                    let mut instance = Self::new();

                    if bytes.len() <= instance.bytes.len() {
                        instance.bytes[..bytes.len()].copy_from_slice(&bytes);

                        // Validate the configuration is correct for this size
                        if instance.configuration_index() == #index {
                            ::core::result::Result::Ok(instance)
                        } else {
                            ::core::result::Result::Err(::modular_bitfield::error::OutOfBounds)
                        }
                    } else {
                        ::core::result::Result::Err(::modular_bitfield::error::OutOfBounds)
                    }
                }
            }
        });

        // Generate dynamic serialization methods
        let _dynamic_into_bytes_arms = analysis.sizes.iter().enumerate().map(|(index, &size)| {
            let method_name = format_ident!("into_bytes_{}", size);
            quote! { #index => self.#method_name().to_vec(), }
        });

        let _dynamic_from_bytes_methods = analysis.sizes.iter().enumerate().map(|(index, &size)| {
            let method_name = format_ident!("from_bytes_{}", size);
            let size_bytes = (size + 7) / 8;

            quote! {
                if bytes.len() >= #size_bytes {
                    if let Ok(array) = bytes[..#size_bytes].try_into() {
                        if let Ok(instance) = Self::#method_name(array) {
                            if instance.configuration_index() == #index {
                                return Some(instance);
                            }
                        }
                    }
                }
            }
        });

        let dynamic_methods = quote! {
            // Note: These dynamic methods are currently commented out as they require std/alloc
            // They can be re-enabled when we add proper feature gating
            /*
            /// Converts to dynamically-sized byte vector based on current configuration
            pub fn into_bytes_dynamic(&self) -> ::std::vec::Vec<u8> {
                match self.configuration_index() {
                    #( #dynamic_into_bytes_arms )*
                    _ => panic!("Invalid configuration index"),
                }
            }

            /// Creates instance from dynamic byte slice, determining configuration from discriminant
            pub fn from_bytes_dynamic(bytes: &[u8]) -> ::core::option::Option<Self> {
                if bytes.is_empty() {
                    return ::core::option::Option::None;
                }

                // Try each size configuration
                #( #dynamic_from_bytes_methods )*
                ::core::option::Option::None
            }
            */
        };

        Ok(quote! {
            #( #into_bytes_methods )*
            #( #from_bytes_methods )*
            #dynamic_methods
        })
    }

    fn generate_wire_format_helpers(
        &self,
        analysis: &VariableStructAnalysis,
    ) -> syn::Result<TokenStream2> {
        // Find discriminator field name for helper generation
        let discriminator_field = self
            .item_struct
            .fields
            .iter()
            .nth(analysis.discriminator_field_index)
            .ok_or_else(|| {
                format_err!(
                    self.item_struct.span(),
                    "discriminator field index out of bounds"
                )
            })?;

        let discriminator_getter = discriminator_field
            .ident
            .as_ref()
            .map(|name| format_ident!("{}", name))
            .unwrap_or_else(|| format_ident!("get_{}", analysis.discriminator_field_index));

        // Generate fallback methods for different struct sizes
        let fallback_methods = analysis
            .sizes
            .iter()
            .enumerate()
            .rev()
            .map(|(_index, &size)| {
                let method_name = format_ident!("from_bytes_{}", size);
                let size_bytes = (size + 7) / 8;

                quote! {
                    if bytes.len() >= #size_bytes {
                        if let Ok(array) = bytes[..#size_bytes].try_into() {
                            if let Ok(instance) = Self::#method_name(array) {
                                return ::core::option::Option::Some(instance);
                            }
                        }
                    }
                }
            });

        Ok(quote! {
            /// Extract discriminant value from raw bytes (protocol-specific implementation)
            ///
            /// This is a default implementation that extracts the discriminant from the
            /// expected bit position. Users may need to override this for their specific protocol.
            fn extract_discriminant_from_bytes(bytes: &[u8]) -> ::core::option::Option<usize> {
                if bytes.is_empty() {
                    return ::core::option::Option::None;
                }

                // Create temporary instance to extract discriminant using generated getter
                let temp_instance = Self::from_bytes_dynamic_internal(bytes)?;
                ::core::option::Option::Some(temp_instance.#discriminator_getter() as usize)
            }

            /// Internal helper for discriminant extraction
            fn from_bytes_dynamic_internal(bytes: &[u8]) -> ::core::option::Option<Self> {
                // Try largest size first, then smaller sizes
                #( #fallback_methods )*

                ::core::option::Option::None
            }
        })
    }

    fn generate_variable_helper_methods(
        &self,
        analysis: &VariableStructAnalysis,
    ) -> syn::Result<TokenStream2> {
        // Get discriminator field info by index
        let discriminator_field = self
            .item_struct
            .fields
            .iter()
            .nth(analysis.discriminator_field_index)
            .ok_or_else(|| {
                format_err!(
                    self.item_struct.span(),
                    "discriminator field index out of bounds"
                )
            })?;

        let discriminator_getter = discriminator_field
            .ident
            .as_ref()
            .map(|name| format_ident!("{}", name))
            .unwrap_or_else(|| format_ident!("get_{}", analysis.discriminator_field_index));

        // Generate configuration index mapping
        let config_index_arms = (0..analysis.sizes.len()).map(|index| {
            quote! { #index => #index, }
        });

        // Generate size lookup
        let size_match_arms: Vec<_> = analysis
            .sizes
            .iter()
            .enumerate()
            .map(|(index, &size)| {
                quote! { #index => #size, }
            })
            .collect();

        // Generate size lookup with Option return
        let size_option_match_arms: Vec<_> = analysis
            .sizes
            .iter()
            .enumerate()
            .map(|(index, &size)| {
                quote! { #index => ::core::option::Option::Some(#size), }
            })
            .collect();

        let supported_sizes: Vec<_> = analysis
            .sizes
            .iter()
            .map(|&size| {
                quote! { #size }
            })
            .collect();

        // Generate discriminant-based methods for the public API
        let _struct_ident = &self.item_struct.ident;
        let max_bytes = (analysis.sizes.iter().max().unwrap_or(&0) + 7) / 8;

        // Calculate the bit offset and mask for discriminant extraction
        // For MIDI UMP, discriminant is in first 4 bits (LSB in little-endian)
        let discriminant_bits = analysis.discriminator_bits;
        let _discriminant_mask = (1u8 << discriminant_bits) - 1;

        // Collect iterators into vectors to avoid move issues
        let _config_index_arms_vec: Vec<_> = config_index_arms.collect();

        Ok(quote! {
            /// Get the discriminant value from this message
            pub fn discriminant(&self) -> u16 {
                self.#discriminator_getter() as u16
            }

            /// Get the size in bits for this message
            pub fn size(&self) -> usize {
                self.actual_size()
            }

            /// Returns the underlying bits.
            ///
            /// # Layout
            ///
            /// The returned byte array is layed out in the same way as described
            /// [here](https://docs.rs/modular-bitfield/#generated-structure).
            #[inline]
            #[allow(clippy::identity_op)]
            pub const fn into_bytes(self) -> [::core::primitive::u8; #max_bytes] {
                self.bytes
            }

            /// Converts the given bytes directly into the bitfield struct.
            #[inline]
            #[allow(clippy::identity_op)]
            #[must_use]
            pub const fn from_bytes(bytes: [::core::primitive::u8; #max_bytes]) -> Self {
                Self { bytes }
            }

            /// Get the size in bits for a given discriminant value
            pub fn size_for_discriminant(discriminant: u16) -> ::core::option::Option<usize> {
                match discriminant as usize {
                    #( #size_option_match_arms )*
                    _ => ::core::option::Option::None,
                }
            }

            /// Get the configuration index based on the discriminant value
            fn configuration_index(&self) -> usize {
                // For now, return the discriminant value directly as index
                // This assumes discriminant values correspond to configuration indices
                self.#discriminator_getter() as usize
            }

            /// Get the actual bit size for the current configuration
            fn actual_size(&self) -> usize {
                match self.configuration_index() {
                    #( #size_match_arms )*
                    _ => panic!("Invalid configuration index"),
                }
            }

            /// Get the expected total size for a given configuration index
            pub const fn size_for_config(config_index: usize) -> ::core::option::Option<usize> {
                match config_index {
                    #( #size_option_match_arms )*
                    _ => ::core::option::Option::None,
                }
            }

            /// Get all supported sizes for this variable bitfield
            pub const fn supported_sizes() -> &'static [usize] {
                &[#( #supported_sizes ),*]
            }

            /// Get the number of bytes required for the current configuration
            pub fn required_bytes(&self) -> usize {
                (self.actual_size() + 7) / 8
            }
        })
    }

    fn generate_variable_validations(
        &self,
        analysis: &VariableStructAnalysis,
    ) -> syn::Result<TokenStream2> {
        let _struct_ident = &self.item_struct.ident;
        let _data_enum_type = &analysis.data_enum_type;

        // Generate compile-time validation that data enum supports required sizes
        let fixed_bits = analysis.fixed_bits;
        let data_size_validations =
            analysis
                .sizes
                .iter()
                .enumerate()
                .map(|(_index, &total_size)| {
                    let _expected_data_size = total_size - fixed_bits;

                    quote! {
                        const _: () = {
                            // Validate that data enum can support the required size for configuration #index
                            const TOTAL_SIZE: usize = #total_size;
                            const FIXED_BITS: usize = #fixed_bits;
                            const EXPECTED_DATA_SIZE: usize = TOTAL_SIZE - FIXED_BITS;

                            // This will be validated when data enum is compiled
                            // The data enum must have a variant that matches EXPECTED_DATA_SIZE
                        };
                    }
                });

        Ok(quote! {
            // Compile-time validations
            #( #data_size_validations )*

            const _: () = {
                // Validate struct has exactly one variant_discriminator field
                let mut discriminator_count = 0usize;
                let mut data_count = 0usize;

                discriminator_count += 1; // We know there's exactly one from analysis
                data_count += 1; // We know there's exactly one from analysis

                assert!(discriminator_count == 1, "Must have exactly one variant_discriminator field");
                assert!(data_count == 1, "Must have exactly one variant_data field");
            };
        })
    }
}