use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Path, PathSegment, Type, TypePath};

extern crate proc_macro;

#[proc_macro_derive(Tag)]
pub fn tag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = match input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut start = Vec::new();
            let mut end = Vec::new();
            for variant in variants {
                if variant.fields.len() != 1 {
                    panic!("Tag is only supported for enum values with exactly one field");
                }
                let variant_name = &variant.ident;
                start.push(quote_spanned! {name.span()=>
                    #name::#variant_name(inner) => inner.start()
                });
                end.push(quote_spanned! {name.span()=>
                    #name::#variant_name(inner) => inner.end()
                });
            }
            quote! {
                impl Tag for #name {
                    fn start(&self) -> Position {
                        return match self {
                            #(#start,)*
                        };
                    }

                    fn end(&self) -> Position {
                        return match self {
                            #(#start,)*
                        };
                    }
                }
            }
        }
        _ => panic!("derive(Tag) is only supported for enums"),
    };
    return proc_macro::TokenStream::from(expanded);
}

#[proc_macro_derive(DocumentNode)]
pub fn document_node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let range_fields: Vec<&syn::Field> = named
                .iter()
                .filter(|field| {
                    get_type(&field.ty)
                        .and_then(|r#type| get_type_name(r#type))
                        .is_some_and(|name| name == "Range")
                })
                .collect();
            match range_fields.len() {
                1 => {
                    let field = &range_fields[0].ident;
                    quote! {
                        impl DocumentNode for #name {
                            fn range(&self) -> Range {
                                return self.#field.clone();
                            }
                        }
                    }
                }
                _ => quote! {
                    impl DocumentNode for #name {
                        fn range(&self) -> Range {
                            let start = self.open_location().start();
                            let end = self.close_location().end();
                            return Range { start, end };
                        }
                    }
                },
            }
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut range = Vec::new();
            for variant in variants {
                if variant.fields.len() != 1 {
                    panic!("DocumentNode is only supported for enum values with exactly one field");
                }
                let variant_name = &variant.ident;
                range.push(quote_spanned! {name.span()=>
                    #name::#variant_name(inner) => inner.range()
                });
            }
            quote! {
                impl DocumentNode for #name {
                    fn range(&self) -> Range {
                        return match self {
                            #(#range,)*
                        };
                    }
                }
            }
        }
        _ => panic!("derive(DocumentNode) is only supported for structs and enums"),
    };
    return proc_macro::TokenStream::from(expanded);
}

#[proc_macro_derive(ParsableTag)]
pub fn parsable_tag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = match input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut definition = Vec::new();
            let mut open_location = Vec::new();
            let mut close_location = Vec::new();
            let mut body = Vec::new();
            let mut spel_attribute = Vec::new();
            let mut spel_attributes = Vec::new();
            for variant in variants {
                if variant.fields.len() != 1 {
                    panic!("ParseTag is only supported for enum values with exactly one field");
                }
                let name = &variant.ident;
                definition.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.definition()
                });
                open_location.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.open_location()
                });
                close_location.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.close_location()
                });
                body.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.body()
                });
                spel_attribute.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.spel_attribute(name)
                });
                spel_attributes.push(quote_spanned! {name.span()=>
                    SpmlTag::#name(inner) => inner.spel_attributes()
                });
            }
            quote! {
                impl ParsableTag for #name {
                    fn parse(parser: &mut TreeParser) -> Result<ParsedTag<Self>> {
                        unimplemented!();
                    }

                    fn definition(&self) -> TagDefinition {
                        return match self {
                            #(#definition,)*
                        };
                    }

                    fn open_location(&self) -> &SingleLineLocation {
                        return match self {
                            #(#open_location,)*
                        };
                    }

                    fn close_location(&self) -> &SingleLineLocation {
                        return match self {
                            #(#close_location,)*
                        };
                    }

                    fn body(&self) -> &Option<TagBody> {
                        return match self {
                            #(#body,)*
                        };
                    }

                    fn spel_attributes(&self) -> Vec<(&str, &ParsedAttribute<SpelAttribute>)> {
                        return match self {
                            #(#spel_attributes,)*
                        };
                    }

                    fn spel_attribute(&self, name: &str) -> Option<&ParsedAttribute<SpelAttribute>> {
                        return match self {
                            #(#spel_attribute,)*
                        };
                    }
                }
            }
        }
        _ => panic!("derive(ParsableTag) is only supported for structs and enums"),
    };
    return proc_macro::TokenStream::from(expanded);
}

fn get_type(r#type: &Type) -> Option<&PathSegment> {
    return match r#type {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments.last(),
        _ => None,
    };
}

fn get_type_name(r#type: &PathSegment) -> Option<String> {
    r#type.ident.span().source_text()
}
