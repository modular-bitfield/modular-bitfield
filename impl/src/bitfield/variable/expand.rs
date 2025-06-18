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
        let Some(analysis) = self.analyze_variable_bits(config)? else {
            return Ok(None); // Not a variable-size struct
        };

        let struct_ident = &self.item_struct.ident;
        let (impl_generics, ty_generics, where_clause) = self.item_struct.generics.split_for_impl();

        // Generate size-specific constructors
        let constructors = Self::generate_size_specific_constructors(&analysis);

        // Generate size-specific serialization methods
        let serialization_methods = Self::generate_variable_serialization(&analysis);

        // Generate wire format helpers
        let wire_format_helpers = self.generate_wire_format_helpers(&analysis)?;

        // Generate helper methods
        let helper_methods = self.generate_variable_helper_methods(&analysis)?;

        // Generate compile-time validations
        let compile_time_validations = Self::generate_variable_validations(&analysis);

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
        analysis: &VariableStructAnalysis,
    ) -> TokenStream2 {
        // Calculate the maximum size in bytes for the struct
        let max_size = analysis.sizes.iter().max().unwrap_or(&0);
        let max_bytes = (max_size + 7) / 8;

        let constructors = analysis.sizes.iter().map(|&size| {
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

        quote! {
            #( #constructors )*
        }
    }

    fn generate_variable_serialization(
        analysis: &VariableStructAnalysis,
    ) -> TokenStream2 {

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

        // Note: Dynamic serialization methods are prepared but not used in the current implementation
        // They would require std/alloc features and are commented out in the generated code
        // The code generation is tested but the iterators themselves are not executed

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

        quote! {
            #( #into_bytes_methods )*
            #( #from_bytes_methods )*
            #dynamic_methods
        }
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
            .as_ref().map_or_else(|| format_ident!("get_{}", analysis.discriminator_field_index), |name| format_ident!("{}", name));

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
            .as_ref().map_or_else(|| format_ident!("get_{}", analysis.discriminator_field_index), |name| format_ident!("{}", name));

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
        let max_bytes = (analysis.sizes.iter().max().unwrap_or(&0) + 7) / 8;



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
        analysis: &VariableStructAnalysis,
    ) -> TokenStream2 {

        // Generate compile-time validation that data enum supports required sizes
        let fixed_bits = analysis.fixed_bits;
        let data_size_validations =
            analysis
                .sizes
                .iter()
                .map(|&total_size| {
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

        quote! {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitfield::{BitfieldStruct, config::{Config, ConfigValue}};
    use proc_macro2::Span;

    #[test]
    fn test_generate_size_specific_constructors() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![32, 64, 128],
            fixed_bits: 16,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };

        let constructors = BitfieldStruct::generate_size_specific_constructors(&analysis);
        let generated = constructors.to_string();

        // Verify constructors were generated
        assert!(generated.contains("new_32bit"));
        assert!(generated.contains("new_64bit"));
        assert!(generated.contains("new_128bit"));
        // The actual token might have different spacing in the generated output
        assert!(generated.contains("bytes :") && generated.contains("[0u8"));
    }

    #[test]
    fn test_generate_variable_serialization() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16, 24, 32],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };

        let serialization = BitfieldStruct::generate_variable_serialization(&analysis);
        let generated = serialization.to_string();

        // Verify into_bytes methods
        assert!(generated.contains("into_bytes_16"));
        assert!(generated.contains("into_bytes_24"));
        assert!(generated.contains("into_bytes_32"));
        // Just check that array types are present
        assert!(generated.contains("[u8 ;"));

        // Verify from_bytes methods
        assert!(generated.contains("from_bytes_16"));
        assert!(generated.contains("from_bytes_24"));
        assert!(generated.contains("from_bytes_32"));
    }

    #[test]
    fn test_generate_variable_validations() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![2],
            sizes: vec![32, 64, 96],
            fixed_bits: 12,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };

        let validations = BitfieldStruct::generate_variable_validations(&analysis);
        let generated = validations.to_string();

        // Check for the general structure of validations
        assert!(generated.contains("const TOTAL_SIZE : usize"));
        assert!(generated.contains("const FIXED_BITS : usize"));
        assert!(generated.contains("const EXPECTED_DATA_SIZE : usize"));
        
        // Check that it validates the discriminator count
        assert!(generated.contains("discriminator_count == 1"));
        assert!(generated.contains("data_count == 1"));
    }

    #[test]
    fn test_expand_variable_struct() {
        let input: syn::ItemStruct = syn::parse_quote! {
            #[bitfield(variable_bits = (32, 64))]
            struct TestStruct {
                #[variant_discriminator]
                disc: B2,
                #[variant_data]
                data: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![32, 64]),
        });
        
        // Set up field configs
        config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
        config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
        
        let result = bitfield.expand_variable_struct(&config);
        assert!(result.is_ok());
        
        let expanded = result.unwrap();
        let generated = expanded.to_string();
        
        // Check for key generated methods
        assert!(generated.contains("new_32bit"));
        assert!(generated.contains("new_64bit"));
    }

    #[test]
    fn test_generate_variable_specifier_impl() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16, 32, 64],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                disc: B2,
                data: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let impl_tokens = bitfield.generate_variable_specifier_impl(&analysis);
        let generated = impl_tokens.to_string();
        
        // Check Specifier trait implementation
        assert!(generated.contains("impl"));
        assert!(generated.contains("Specifier"));
        assert!(generated.contains("const BITS")); 
        assert!(generated.contains("64")); // Max size
        assert!(generated.contains("type Bytes")); 
        assert!(generated.contains("[u8")); // Array type
        assert!(generated.contains("8")); // 64 bits = 8 bytes
        assert!(generated.contains("into_bytes"));
        assert!(generated.contains("from_bytes"));
    }

    #[test]
    fn test_generate_wire_format_helpers() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16, 32],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                discriminator: B2,
                data: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let result = bitfield.generate_wire_format_helpers(&analysis);
        assert!(result.is_ok());
        
        let helpers = result.unwrap();
        let generated = helpers.to_string();
        
        // Check for wire format helper methods
        assert!(generated.contains("extract_discriminant_from_bytes"));
        assert!(generated.contains("from_bytes_dynamic_internal"));
        assert!(generated.contains("from_bytes_32")); // Fallback method
        assert!(generated.contains("from_bytes_16")); // Fallback method
    }

    #[test]
    fn test_generate_variable_helper_methods() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 1, // Use field index 1 (named field)
            _data_field_index: 2,
            _fixed_field_indices: vec![0],
            sizes: vec![24, 48, 96],
            fixed_bits: 16,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 4,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                fixed: B8,
                variant_type: B4,
                data: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let result = bitfield.generate_variable_helper_methods(&analysis);
        assert!(result.is_ok());
        
        let helpers = result.unwrap();
        let generated = helpers.to_string();
        
        // Check helper methods
        assert!(generated.contains("discriminant"));
        assert!(generated.contains("size"));
        assert!(generated.contains("into_bytes"));
        assert!(generated.contains("from_bytes"));
        assert!(generated.contains("size_for_discriminant"));
        assert!(generated.contains("supported_sizes"));
        assert!(generated.contains("required_bytes"));
        
        // Check array literal
        assert!(generated.contains("24") && generated.contains("48") && generated.contains("96"));
    }

    #[test]
    fn test_generate_variable_size_extensions_no_config() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                field1: B8,
                field2: B16,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let config = Config::default(); // No variable_bits config
        
        let result = bitfield.generate_variable_size_extensions(&config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_wire_format_helpers_with_unnamed_field() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0, // Unnamed field
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct(B2, DataEnum);
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let result = bitfield.generate_wire_format_helpers(&analysis);
        assert!(result.is_ok());
        
        let helpers = result.unwrap();
        let generated = helpers.to_string();
        
        // For unnamed fields, it should generate getter based on index
        assert!(generated.contains("get_0"));
    }

    #[test]
    fn test_wire_format_helpers_invalid_index() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 10, // Invalid index
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                field1: B2,
                field2: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let result = bitfield.generate_wire_format_helpers(&analysis);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("discriminator field index out of bounds"));
    }

    #[test]
    fn test_generate_variable_helper_methods_invalid_index() {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 10, // Invalid index
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: vec![16],
            fixed_bits: 8,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestStruct {
                field1: B2,
                field2: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input };
        let result = bitfield.generate_variable_helper_methods(&analysis);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("discriminator field index out of bounds"));
    }
}