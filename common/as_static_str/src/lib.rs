use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::{parse_macro_input, AttributeArgs, Lit, Meta, NestedMeta};

enum Style {
    SameAsName,
    Title,
    Snake,
}

use Style::*;

#[proc_macro_attribute]
/// Produce a `as_static_str` function for the Enum.
///
/// Optionally implement `Debug`, `Display` and `Slog::Value` using the output of this function.
/// Optionally implement too `variants_as_static_str` to get an array slice with the
/// representations of each variant of the enum, as defined by `as_static_str`.
pub fn as_static_str(args: TokenStream, enum_code: TokenStream) -> TokenStream {
    // decide what we want to implement from Debug, Display, and Slog::Value
    let mut impl_debug = false;
    let mut impl_display = false;
    let mut impl_slog_value = false;
    // check if implement `variants_as_static_str` too
    let mut impl_variant_list = false;

    let mut style = SameAsName;

    let enum_code: DeriveInput = syn::parse(enum_code).unwrap();
    let args = parse_macro_input!(args as AttributeArgs);
    for arg in args {
        if let NestedMeta::Meta(meta_path) = arg {
            match meta_path {
                Meta::Path(path) => {
                    if path.is_ident("Debug") {
                        impl_debug = true;
                    } else if path.is_ident("Display") {
                        impl_display = true;
                    } else if path.is_ident("Slog") {
                        impl_slog_value = true;
                    } else if path.is_ident("VariantList") {
                        impl_variant_list = true;
                    } else {
                        panic!("Bad macro attribute. Only Debug, Display, Slog or VariantList")
                    }
                }
                Meta::NameValue(key_val) => {
                    if key_val.path.is_ident("style") {
                        if let Lit::Str(val) = key_val.lit {
                            match val.value().as_str() {
                                "title" => style = Title,
                                "snake" => style = Snake,
                                "default" => style = SameAsName,
                                other => panic!("Unsuported style {}", other),
                            }
                        }
                    } else {
                        panic!("Bad macro attribute, use `style = \"some_style\"`")
                    }
                }
                _ => {
                    panic!("Bad macro attribute. Only Debug, Display, Slog or VariantList")
                }
            }
        } else {
        }
    }

    // type name to build the match arms. The `MyEnum` in `MyEnum::MyVariant`
    let enum_name = &enum_code.ident;

    // generics and parameters
    let (impl_generics, ty_generics, where_clause) = &enum_code.generics.split_for_impl();

    if let syn::Data::Enum(the_enum) = &enum_code.data {
        let mut variants = Vec::with_capacity(the_enum.variants.iter().count());
        // Build the arms of the match expression
        let per_variant_name = the_enum.variants.iter().map(|v| {
            let variant_name = &v.ident;
            // build the title repr
            let variant_repr = match style {
                SameAsName => quote! { stringify!(#variant_name) },
                Title => {
                    let title = v
                        .ident
                        .to_string()
                        .chars()
                        .fold((true, String::new()), |(up, mut acc), c| {
                            if c.is_ascii_uppercase() && !up {
                                acc.push(' ');
                            }
                            let up = c.is_ascii_uppercase();
                            acc.push(c);
                            (up, acc)
                        })
                        .1;
                    quote! { #title }
                }
                Snake => {
                    let snake = v
                        .ident
                        .to_string()
                        .chars()
                        .fold((true, String::new()), |(up, mut acc), c| {
                            if c.is_ascii_uppercase() && !up {
                                acc.push('_');
                            }
                            let up = c.is_ascii_uppercase();
                            acc.push(c.to_ascii_lowercase());
                            (up, acc)
                        })
                        .1;
                    quote! { #snake }
                }
            };
            variants.push(variant_repr.clone());
            // Regardless of the form of the variant (Struct, Unit or Tuple) we can use the
            // `MyEnum::MyVariant{ .. }` syntax
            quote! {
                #enum_name::#variant_name{ .. } => #variant_repr
            }
        });

        // add the code that defines the enum
        // add the `as_static_str` fn.
        let gen = quote! {
            #enum_code
            impl #impl_generics #enum_name #ty_generics #where_clause {
                pub fn as_static_str(&self) -> &'static str {
                    match self {
                        #(#per_variant_name),*
                    }
                }

            }
        };

        let mut impls = vec![gen];

        if impl_variant_list {
            // The array of variant representations
            let quoted = quote! {
                impl #impl_generics #enum_name #ty_generics #where_clause {
                    pub fn variants_as_static_str() -> &'static [&'static str] {
                        &[#(#variants),*]
                    }
                }

            };
            impls.push(quoted);
        }

        // impl `std::fmt::Debug`
        if impl_debug {
            let quoted = quote! {
                impl #impl_generics std::fmt::Debug for #enum_name #ty_generics #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(self.as_static_str())
                    }
                }
            };
            impls.push(quoted);
        }

        // impl `std::fmt::Display`
        if impl_display {
            let quoted = quote! {
                impl #impl_generics std::fmt::Display for #enum_name #ty_generics #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(self.as_static_str())
                    }
                }
            };
            impls.push(quoted);
        }

        // impl `slog::Value`
        if impl_slog_value {
            let quoted = quote! {
                impl #impl_generics slog::Value for #enum_name #ty_generics #where_clause {
                    fn serialize(
                        &self,
                        record: &slog::Record,
                        key: slog::Key,
                        serializer: &mut dyn slog::Serializer,
                    ) -> slog::Result {
                        serializer.emit_str(key, &self.as_static_str())
                    }
                }
            };
            impls.push(quoted);
        }

        (quote! {
            #(
                #impls
            )*
        })
        .into()
    } else {
        panic!("Just for enums!")
    }
}
