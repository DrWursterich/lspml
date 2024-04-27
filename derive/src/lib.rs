use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, AngleBracketedGenericArguments, DeriveInput, Expr,
    Field, GenericArgument, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

extern crate proc_macro;

#[proc_macro_derive(ParsableTag, attributes(tag_definition))]
pub fn parsable_tag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut params: Option<Expr> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("tag_definition") {
            params = Some(attr.parse_args().unwrap());
        }
    }
    let definition = params.unwrap();
    let name = input.ident;
    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let plain_attribute_fields: Vec<&syn::Ident> =
                attribute_fields_with_option_value(&named, "PlainAttribute");
            let spel_attribute_fields: Vec<&syn::Ident> =
                attribute_fields_with_option_value(&named, "SpelAttribute");
            let expanded = quote! {
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
                                return Err(anyhow::anyhow!("sp:attribute tag is unclosed"));
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
                                ">" => body = Some(parser.parse_tag_body()?),
                                _ => (),
                            };
                        }
                        let close_location = node_location(parser.cursor.node());
                        return Ok(Self {
                            open_location,
                            #(#plain_attribute_fields,)*
                            #(#spel_attribute_fields,)*
                            body,
                            close_location,
                        });
                    }
                }
            };
            return proc_macro::TokenStream::from(expanded);
        }
        _ => panic!("requires struct with named fields!"),
    };
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
