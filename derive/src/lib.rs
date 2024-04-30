use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, AngleBracketedGenericArguments, DeriveInput, Expr,
    Field, GenericArgument, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

extern crate proc_macro;

#[proc_macro_derive(ParsableTag, attributes(tag_definition))]
pub fn parsable_tag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let mut params: Option<Expr> = None;
            for attr in &input.attrs {
                if attr.path().is_ident("tag_definition") {
                    params = Some(attr.parse_args().unwrap());
                }
            }
            let definition = params.unwrap();
            let plain_attribute_fields: Vec<&syn::Ident> =
                attribute_fields_with_option_value(&named, "PlainAttribute");
            let spel_attribute_fields: Vec<&syn::Ident> =
                attribute_fields_with_option_value(&named, "SpelAttribute");
            quote! {
                impl ParsableTag for #name {
                    fn parse(parser: &mut TreeParser) -> Result<Self> {
                        if !parser.cursor.goto_first_child() {
                            return Err(anyhow::anyhow!("tag is empty"));
                        }
                        let open_location = node_location(parser.cursor.node());
                        #(let mut #plain_attribute_fields = None;)*
                        #(let mut #spel_attribute_fields = None;)*
                        let mut body = None;
                        loop {
                            if !parser.cursor.goto_next_sibling() {
                                return Err(anyhow::anyhow!("{} tag is unclosed", #definition.name));
                            }
                            let node = parser.cursor.node();
                            match node.kind() {
                                "comment" | "xml_comment" => (),
                                #(
                                    stringify!(#plain_attribute_fields) => #plain_attribute_fields =
                                        Some(parser.parse_plain_attribute()?.1),
                                )*
                                #(
                                    stringify!(#spel_attribute_fields) => #spel_attribute_fields =
                                        Some(stringify!(#spel_attribute_fields)
                                            .strip_suffix("_attribute")
                                            .and_then(|n| #definition.attributes.get_by_name(n))
                                            .map(|d| parser.parse_spel_attribute(&d.r#type))
                                            .unwrap()?
                                            .1),
                                )*
                                "self_closing_tag_end" => break,
                                ">" => {
                                    body = Some(parser.parse_tag_body()?);
                                    break;
                                },
                                _ => (),
                            };
                        }
                        let close_location = node_location(parser.cursor.node());
                        parser.cursor.goto_parent();
                        return Ok(Self {
                            open_location,
                            #(#plain_attribute_fields,)*
                            #(#spel_attribute_fields,)*
                            body,
                            close_location,
                        });
                    }

                    fn definition(&self) -> TagDefinition {
                        return #definition;
                    }

                    fn open_location(&self) -> &Location {
                        return &self.open_location;
                    }

                    fn close_location(&self) -> &Location {
                        return &self.close_location;
                    }

                    fn body(&self) -> &Option<TagBody> {
                        return &self.body;
                    }

                    fn range(&self) -> lsp_types::Range {
                        return lsp_types::Range {
                            start: lsp_types::Position {
                                line: self.open_location.line as u32,
                                character: self.open_location.char as u32,
                            },
                            end: lsp_types::Position {
                                line: self.close_location.line as u32,
                                character: (self.close_location.char + self.close_location.length)
                                    as u32,
                            },
                        }
                    }

                    fn spel_attributes(&self) -> Vec<(&str, &SpelAttribute)> {
                        let mut attributes = Vec::new();
                        #(
                            match self.#spel_attribute_fields.as_ref() {
                                Some(field) => attributes.push(
                                    (stringify!(#spel_attribute_fields), field)
                                ),
                                None => (),
                            };
                        )*
                        return attributes;
                    }

                    fn spel_attribute(&self, name: &str) -> Option<&SpelAttribute> {
                        return match format!("{}_attribute", name).as_str() {
                            #(
                                stringify!(#spel_attribute_fields) =>
                                    self.#spel_attribute_fields.as_ref(),
                            )*
                            _ => None,
                        };
                    }
                }
            }
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut definition = Vec::new();
            let mut open_location = Vec::new();
            let mut close_location = Vec::new();
            let mut body = Vec::new();
            let mut range = Vec::new();
            let mut spel_attribute = Vec::new();
            let mut spel_attributes = Vec::new();
            for variant in variants {
                if variant.fields.len() != 1 {
                    panic!("ParseTag is only supported for enum values with exactly one field");
                }
                let name = &variant.ident;
                definition.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.definition()
                });
                open_location.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.open_location()
                });
                close_location.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.close_location()
                });
                body.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.body()
                });
                range.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.range()
                });
                spel_attribute.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.spel_attribute(name)
                });
                spel_attributes.push(quote_spanned! {name.span()=>
                    Tag::#name(inner) => inner.spel_attributes()
                });
            }
            quote! {
                impl ParsableTag for #name {
                    fn parse(parser: &mut TreeParser) -> Result<Self> {
                        unimplemented!();
                    }

                    fn definition(&self) -> TagDefinition {
                        return match self {
                            #(#definition,)*
                        };
                    }

                    fn open_location(&self) -> &Location {
                        return match self {
                            #(#open_location,)*
                        };
                    }

                    fn close_location(&self) -> &Location {
                        return match self {
                            #(#close_location,)*
                        };
                    }

                    fn body(&self) -> &Option<TagBody> {
                        return match self {
                            #(#body,)*
                        };
                    }

                    fn range(&self) -> lsp_types::Range {
                        return match self {
                            #(#range,)*
                        };
                    }

                    fn spel_attributes(&self) -> Vec<(&str, &SpelAttribute)> {
                        return match self {
                            #(#spel_attributes,)*
                        };
                    }

                    fn spel_attribute(&self, name: &str) -> Option<&SpelAttribute> {
                        return match self {
                            #(#spel_attribute,)*
                        };
                    }
                }
            }
        },
        _ => panic!("derive(ParsableTag) is only supported for structs and enums"),
    };
    return proc_macro::TokenStream::from(expanded);
}

fn attribute_fields_with_option_value<'a, T>(
    fields: &'a Punctuated<Field, T>,
    value_type_name: &str,
) -> Vec<&'a Ident> {
    fields
        .iter()
        .filter(|field| is_type_option_of(field, &value_type_name))
        .filter_map(|field| is_attribute_field(field))
        .collect()
}

fn is_attribute_field(field: &Field) -> Option<&Ident> {
    field.ident.as_ref().filter(|ident| {
        ident
            .span()
            .source_text()
            .is_some_and(|name| name.ends_with("_attribute"))
    })
}

fn is_type_option_of(field: &Field, type_name: &str) -> bool {
    if let Some(r#type) = get_type(&field.ty) {
        if get_type_name(r#type).is_some_and(|name| name == "Option") {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                &r#type.arguments
            {
                if let Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.first()
                {
                    return path.is_ident(type_name);
                }
            }
        }
    }
    return false;
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
