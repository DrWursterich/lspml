use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, AngleBracketedGenericArguments, DeriveInput, Expr,
    Field, GenericArgument, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

extern crate proc_macro;

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
                attribute_fields_with_option_of_parsed_attribute_value(&named, "SpelAttribute");
            quote! {
                impl #name {
                    fn try_parse(
                        parser: &mut TreeParser,
                        depth_counter: &mut DepthCounter,
                    ) -> Result<ParsedTag<Self>> {
                        let parent_node = parser.cursor.node();
                        let mut errors = Vec::new();
                        let mut movement = NodeMovement::FirstChild;
                        depth_counter.bump();
                        let open_location;
                        loop {
                            open_location = match parser.goto(&movement) {
                                NodeMovingResult::NonExistent | NodeMovingResult::Missing(_) => {
                                    return Err(anyhow::anyhow!("tag is empty"));
                                },
                                NodeMovingResult::Erroneous(node) => {
                                    return Ok(ParsedTag::Unparsable(
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node),
                                    ));
                                },
                                NodeMovingResult::Superfluous(node) => {
                                    errors.push(TagError::Superfluous(
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node),
                                    ));
                                    movement = NodeMovement::NextSibling;
                                    continue;
                                },
                                NodeMovingResult::Ok(node) => match node_location(node) {
                                    Location::SingleLine(location) => location,
                                    location => return Ok(ParsedTag::Unparsable(
                                        format!(
                                            "\"{}\" should be on a single line",
                                            parser.node_text(&node)?,
                                        ),
                                        location,
                                    ))
                                },
                            };
                            break;
                        }
                        #(let mut #plain_attribute_fields = None;)*
                        #(let mut #spel_attribute_fields = None;)*
                        let mut body = None;
                        let close_location;
                        loop {
                            close_location = match parser.goto(&NodeMovement::NextSibling) {
                                NodeMovingResult::NonExistent => return Ok(ParsedTag::Unparsable(
                                    format!("\"{}\" tag is unclosed", #definition.name),
                                    node_location(parent_node),
                                )),
                                NodeMovingResult::Missing(node) if node.kind() == ">" => {
                                    body = Some(parser.parse_tag_body()?);
                                    match #name::parse_closing_tag(parser, &mut errors)? {
                                        Ok(location) => location,
                                        Err((text, location)) => return Ok(
                                            ParsedTag::Unparsable(text, location),
                                        ),
                                    }
                                },
                                NodeMovingResult::Missing(node) if node.kind() == "self_closing_tag_end" => {
                                    let (first, last) =
                                        missing_self_closing_tag_first_and_last_possible_location(
                                            node,
                                            parser,
                                        )?;
                                    errors.push(TagError::Missing("/>".to_string(), first));
                                    last
                                },
                                NodeMovingResult::Missing(node) => {
                                    return Ok(ParsedTag::Unparsable(
                                        format!(
                                            "\"{}\" is missing in \"{}\" tag",
                                            node.kind(),
                                            #definition.name
                                        ),
                                        node_location(parent_node),
                                    ));
                                },
                                NodeMovingResult::Erroneous(node) => {
                                    return Ok(ParsedTag::Unparsable(
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node)
                                    ));
                                },
                                NodeMovingResult::Superfluous(node) => {
                                    errors.push(TagError::Superfluous(
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node),
                                    ));
                                    continue;
                                },
                                NodeMovingResult::Ok(node) => match node.kind() {
                                    #(stringify!(#plain_attribute_fields) => {
                                        #plain_attribute_fields = parser.parse_plain_attribute(),
                                        continue;
                                    },)*
                                    #(stringify!(#spel_attribute_fields) => {
                                        #spel_attribute_fields = Some(
                                            stringify!(#spel_attribute_fields)
                                                .strip_suffix("_attribute")
                                                .and_then(|n| #definition.attributes.get_by_name(n))
                                                .map(|d| parser.parse_spel_attribute(&d.r#type))
                                                .unwrap()?,
                                        );
                                        continue;
                                    },)*
                                    "self_closing_tag_end" => node_location(node),
                                    ">" => {
                                        body = Some(parser.parse_tag_body()?);
                                        match #name::parse_closing_tag(parser, &mut errors)? {
                                            Ok(location) => location,
                                            Err((text, location)) => return Ok(
                                                ParsedTag::Unparsable(text, location),
                                            ),
                                        }
                                    },
                                    _ => continue,
                                },
                            };
                            break;
                        }
                        let close_location = match close_location {
                            Location::SingleLine(location) => location,
                            location => return Ok(ParsedTag::Unparsable(
                                format!(
                                    "\"{}\" should be on a single line",
                                    parser.node_text(&parser.cursor.node())?,
                                ),
                                location,
                            ))
                        };
                        let tag = Self {
                            open_location,
                            #(#plain_attribute_fields,)*
                            #(#spel_attribute_fields,)*
                            body,
                            close_location,
                        };
                        return Ok(match errors.is_empty() {
                            true => ParsedTag::Valid(tag),
                            false => ParsedTag::Erroneous(tag, errors),
                        });
                    }

                    fn parse_closing_tag(
                        parser: &mut TreeParser,
                        errors: &mut Vec<TagError>,
                    ) -> Result<Result<Location, (String, Location)>> {
                        loop {
                            return Ok(Ok(match parser.goto(&NodeMovement::Current) {
                                NodeMovingResult::Missing(node) => {
                                    let (first, last) =
                                        missing_close_tags_first_and_last_possible_location(
                                            node,
                                            parser,
                                        )?;
                                    errors.push(TagError::Missing(
                                        format!("</{}>", #definition.name),
                                        first
                                    ));
                                    last
                                },
                                NodeMovingResult::Erroneous(node) => {
                                    return Ok(Err((
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node),
                                    )));
                                },
                                NodeMovingResult::Superfluous(node) => {
                                    errors.push(TagError::Superfluous(
                                        parser.node_text(&node)?.to_string(),
                                        node_location(node),
                                    ));
                                    continue;
                                },
                                NodeMovingResult::Ok(node) => node_location(node),
                                _ => continue,
                            }));
                        }
                    }
                }

                impl ParsableTag for #name {
                    fn parse(parser: &mut TreeParser) -> Result<ParsedTag<Self>> {
                        let mut depth_counter = DepthCounter::new();
                        let result = #name::try_parse(parser, &mut depth_counter);
                        for _ in 0..depth_counter.get() {
                            parser.cursor.goto_parent();
                        }
                        return result;
                    }

                    fn definition(&self) -> TagDefinition {
                        return #definition;
                    }

                    fn open_location(&self) -> &SingleLineLocation {
                        return &self.open_location;
                    }

                    fn close_location(&self) -> &SingleLineLocation {
                        return &self.close_location;
                    }

                    fn body(&self) -> &Option<TagBody> {
                        return &self.body;
                    }

                    fn spel_attributes(&self) -> Vec<(&str, &ParsedAttribute<SpelAttribute>)> {
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

                    fn spel_attribute(&self, name: &str) -> Option<&ParsedAttribute<SpelAttribute>> {
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

fn attribute_fields_with_option_of_parsed_attribute_value<'a, T>(
    fields: &'a Punctuated<Field, T>,
    value_type_name: &str,
) -> Vec<&'a Ident> {
    fields
        .iter()
        .filter(|field| is_type_option_of_parsed_attribute_of(field, &value_type_name))
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
    return get_first_generic_type_of(&field.ty, "Option")
        .and_then(|a| match a {
            Type::Path(TypePath { path, .. }) => Some(path),
            _ => None,
        })
        .is_some_and(|path| path.is_ident(type_name));
}

fn is_type_option_of_parsed_attribute_of(field: &Field, type_name: &str) -> bool {
    return get_first_generic_type_of(&field.ty, "Option")
        .and_then(|r#type| get_first_generic_type_of(r#type, "ParsedAttribute"))
        .and_then(|a| match a {
            Type::Path(TypePath { path, .. }) => Some(path),
            _ => None,
        })
        .is_some_and(|path| path.is_ident(type_name));
}

fn get_first_generic_type_of<'a>(r#type: &'a Type, type_name: &str) -> Option<&'a Type> {
    return get_type(r#type)
        .filter(|r#type| get_type_name(r#type).is_some_and(|name| name == type_name))
        .and_then(get_first_generic_type);
}

fn get_first_generic_type(path: &PathSegment) -> Option<&Type> {
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &path.arguments
    {
        if let Some(GenericArgument::Type(r#type)) = args.first() {
            return Some(r#type);
        }
    }
    return None;
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
