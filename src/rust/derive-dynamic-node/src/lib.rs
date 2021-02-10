#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{parse_quote, Attribute, Data, Field, Fields, Ident, Meta, NestedMeta, Type};
use syn::spanned::Spanned;

const CREATE_TIME: &'static str = "created_time";
const LAST_SEEN_TIME: &'static str = "last_seen_time";
const TERMINATE_TIME: &'static str = "terminated_time";

fn name_and_ty(field: &Field) -> (&Ident, &Type) {
    (field.ident.as_ref().unwrap(), &field.ty)
}

#[proc_macro_derive(DynamicNode)]
pub fn derive_node_description(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_struct = match input.data {
        Data::Struct(input_struct) => input_struct,
        _ => panic!("Only available for struct"),
    };

    let fields = match input_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Requires named fields"),
    };

    let methods = fields
        .iter()
        .map(|field| property_methods(field))
        .fold(quote!(), |mut acc, method| {
            acc.extend(method);
            acc
        });

    let struct_name = &input.ident;
    let struct_name_string = input.ident.to_string();

    let node_name = format!("{}Node", struct_name);
    let node_name = syn::Ident::new(&node_name, struct_name.span());

    let node_trait_name = format!("I{}Node", struct_name);
    let node_trait_name = syn::Ident::new(&node_trait_name, struct_name.span());

    let q = quote!(

        #[derive(Clone, Debug)]
        pub struct #node_name {
            dynamic_node: grapl_graph_descriptions::graph_description::DynamicNode,
        }

        pub trait #node_trait_name {
            fn get_mut_dynamic_node(&mut self) -> &mut DynamicNode;
            fn get_dynamic_node(&self) -> &DynamicNode;

            #methods
        }

        impl #node_name {
            pub fn new(strategy: grapl_graph_descriptions::graph_description::IdStrategy, seen_at: u64) -> Self {
                let mut properties = std::collections::HashMap::with_capacity(1);

                let dynamic_node = DynamicNode {
                    asset_id: None,
                    host_ip: None,
                    hostname: None,
                    node_type: #struct_name_string .to_owned(),
                    id_strategy: vec![strategy],
                    node_key: uuid::Uuid::new_v4().to_string(),
                    properties,
                    seen_at,
                };

                Self { dynamic_node }
            }

            pub fn with_asset_id(&mut self, asset_id: impl Into<Option<String>>) -> &mut Self {
                let asset_id = asset_id.into();
                self.dynamic_node.asset_id = asset_id.clone();
                if let Some(asset_id) = asset_id {
                    self.dynamic_node.properties.insert("asset_id".to_owned(), asset_id.into());
                }
                self
            }

            pub fn get_node_key(&self) -> &str {
                &self.dynamic_node.node_key
            }

            pub fn clone_node_key(&self) -> String {
                self.dynamic_node.node_key.clone()
            }

            pub fn into_dyn_node(self) -> DynamicNode {
                self.dynamic_node
            }
        }

        impl AsRef<grapl_graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn as_ref(&self) -> &DynamicNode {
                &self.dynamic_node
            }
        }

        impl AsMut<grapl_graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn as_mut(&mut self) -> &mut DynamicNode {
                &mut self.dynamic_node
            }
        }

        impl Into<grapl_graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn into(self) -> DynamicNode {
                self.dynamic_node
            }
        }

        impl Into<grapl_graph_descriptions::graph_description::Node> for #node_name {
            fn into(self) -> Node {
                self.dynamic_node.into()
            }
        }


        impl Into<grapl_graph_descriptions::graph_description::Node> for & #node_name {
            fn into(self) -> Node {
                self.dynamic_node.clone().into()
            }
        }


        impl Into<grapl_graph_descriptions::graph_description::Node> for &mut #node_name {
            fn into(self) -> Node {
                self.dynamic_node.clone().into()
            }
        }

    );

    q.into()
}

#[proc_macro_derive(GraplStaticId, attributes(grapl))]
pub fn derive_static_node(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_struct = match input.data {
        Data::Struct(input_struct) => input_struct,
        _ => panic!("Only available for struct"),
    };

    let fields = match input_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Requires named fields"),
    };

    let id_fields = fields
        .iter()
        .filter_map(|field| {
            for attr in &field.attrs {
                if attr.path.segments.is_empty() {
                    return None;
                }

                let id = &attr.path.segments[0].ident;
                if id.to_string() != "grapl" {
                    continue;
                }

                return field.ident.as_ref();
            }

            None
        })
        .fold(quote!(), |mut acc, f| {
            let f = f.to_string();
            acc.extend(quote!(#f .to_string(), ));
            acc
        });

    assert!(id_fields.to_string().len() > 0);

    let struct_name = &input.ident;

    let node_name_str = format!("{}Node", struct_name);
    let node_name = syn::Ident::new(&node_name_str, struct_name.span());
    // Add node name to id
    let q = quote!(

        impl #node_name {
            pub fn static_strategy() -> IdStrategy {
                Static {
                    primary_key_properties: vec![
                        #id_fields
                    ],
                    primary_key_requires_asset_id: false,
                }.into()
            }
        }
    );

    q.into()
}

fn identity_prop_setter(field: &Field, property_name: &Ident) -> TS2 {
    let mut created_time_prop = false;
    let mut last_seen_time_prop = false;
    let mut terminated_time_prop = false;

    for attr in field.attrs.iter() {
        on_grapl_attrs(attr, |meta_attr| {
            created_time_prop |= meta_attr == CREATE_TIME;
            last_seen_time_prop |= meta_attr == LAST_SEEN_TIME;
            terminated_time_prop |= meta_attr == TERMINATE_TIME;
        });
        if [&created_time_prop, &last_seen_time_prop, &terminated_time_prop]
            .iter()
            .any(|b| **b)
        {
            break;
        }
    }

    let ident = match (created_time_prop, last_seen_time_prop, terminated_time_prop) {
        (true, _, _) => syn::Ident::new(&CREATE_TIME, field.span()),
        (_, true ,_) => syn::Ident::new(&LAST_SEEN_TIME, field.span()),
        (_,_ ,true) => syn::Ident::new(&TERMINATE_TIME, field.span()),
        _ => return quote!(),
    };
    quote!(
        let mut self_strategy = mut_self.id_strategy[0].strategy.as_mut().unwrap();
        match self_strategy {
            grapl_graph_descriptions::graph_description::id_strategy::Strategy::Session(
                grapl_graph_descriptions::graph_description::Session{ref mut #ident, ..}
            ) => {
                * #ident = #property_name;
            }
            s => panic!("Can not set timestamps on non-Session strategy {:?}", s)
        }
    )
}

fn property_methods(field: &Field) -> TS2 {
    let (property_name, property_type): (&Ident, &Type) = name_and_ty(field);

    let get_method_name = format!("get_{}", property_name);
    let get_method_name = syn::Ident::new(&get_method_name, property_name.span());

    let with_method_name = format!("with_{}", property_name);
    let with_method_name = syn::Ident::new(&with_method_name, property_name.span());

    let property_name_str = format!("{}", property_name);

    let set_identity_prop = identity_prop_setter(field, property_name);
    let mut implementation: TS2 = quote!();

    let with_method_implementation = quote!(
        fn #with_method_name(&mut self, #property_name: impl Into<#property_type>) -> &mut Self {
            let #property_name = #property_name .into();
            let mut_self = self.get_mut_dynamic_node();

            mut_self.properties.insert(
                #property_name_str .to_string(),
                #property_name .into(),
            );

            #set_identity_prop

            self
        }
    );
    implementation.extend(with_method_implementation);

    // Given the property type, determine:
    // - the method on `property` to call
    // - the type of the above, which will be the return type of the function
    /* N.B. on this implementation:
     *
     * Constructing pass-through getters (type T -> T) is relatively simple,
     * because we don't need to examine T.
     *
     * It's more complex for situations like (type String -> &str) because we
     * need to recognize that we're getting a String while parsing the AST.
     *
     * Since this is the AST, we don't know whether a given type will
     * resolve to String (or whatever).  All we have is some AST type token.
     * We have to say "tokens `String` and std::string::String both get
     * handled the same way" because the AST doesn't know they resolve
     * to the same thing.
     */
    let (return_type, method_ident): (syn::Type, syn::Ident) = match property_type {
        // janky way to get String="fully::qualified::path::Type" given a TypePath
        Type::Path(typepath) => match typepath
            .path
            .segments
            .iter()
            .into_iter()
            .map(|x| x.ident.to_string())
            .collect::<Vec<String>>()
            .join("::")
            .as_ref()
        {
            /* underlying struct field type    maps to this type   via this method on NodeProperty */
            "String" | "std::string::String" => (parse_quote!(&str), parse_quote!(as_str_prop)),
            "u64" => (parse_quote!(u64), parse_quote!(as_uint_prop)),
            "i64" => (parse_quote!(i64), parse_quote!(as_int_prop)),
            // Anything else no-ops out, without implementing a getter.
            _ => return implementation,
        },
        // If you're seeing this panic, then a field on the struct you're deriving
        // doesn't resolve to a TypePath.  That's a corner case, and assuming
        // you don't actually need a getter for it, it can be handled explicitly
        // with a no-op matcher.
        _ => panic!("Tried to dynamically construct getter for unrecognized type!"),
    };

    let get_method_implementation = quote!(
        fn #get_method_name(&self) -> Option<#return_type> {
            let property_result: Option<&NodeProperty> = self.get_dynamic_node().get_property(#property_name_str);

            match property_result {
              Some(ref property) => property. #method_ident(),
              None => None,
            }
        }
    );
    implementation.extend(get_method_implementation);

    implementation
}

#[proc_macro_derive(GraplSessionId, attributes(grapl))]
pub fn derive_grapl_session(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_struct = match input.data {
        Data::Struct(input_struct) => input_struct,
        _ => panic!("Only available for struct"),
    };

    let fields = match input_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Requires named fields"),
    };

    let mut created_time_prop: Option<String> = None;
    let mut last_seen_time_prop: Option<String> = None;
    let mut terminated_time_prop: Option<String> = None;

    for field in fields.iter() {
        set_timestamp_from_meta(field, CREATE_TIME, &mut created_time_prop);
        set_timestamp_from_meta(field, LAST_SEEN_TIME, &mut last_seen_time_prop);
        set_timestamp_from_meta(field, TERMINATE_TIME, &mut terminated_time_prop);
    }

    if created_time_prop.is_none() {
        panic!("Must set created_time for at least one field");
    }

    if last_seen_time_prop.is_none() {
        panic!("Must set last_seen_time for at least one field");
    }

    if terminated_time_prop.is_none() {
        panic!("Must set terminated_time for at least one field");
    }
    let mut id_fields = quote!();
    for field in fields {
        for attr in &field.attrs {
            on_grapl_attrs(attr, |meta_attr| {
                if meta_attr == "pseudo_key" {
                    let f = field
                        .ident
                        .as_ref()
                        .expect("field is missing an identifier")
                        .to_string();
                    id_fields.extend(quote!(#f .to_string(), ));
                }
            });
        }
    }

    let struct_name = &input.ident;

    let node_name_str = format!("{}Node", struct_name);
    let node_name = syn::Ident::new(&node_name_str, struct_name.span());
    // Add node name to id
    let q = quote!(
        impl #node_name {
            pub fn session_strategy() -> IdStrategy {
                Session {
                    created_time: 0,
                    last_seen_time: 0,
                    terminated_time: 0,
                    primary_key_requires_asset_id: false,
                    primary_key_properties: vec![
                        #id_fields
                    ],
                }.into()
            }

            pub fn identity_strategy() -> IdStrategy {
                return #node_name :: session_strategy()
            }
        }
    );

    q.into()
}

fn on_grapl_attrs(attr: &Attribute, mut on: impl FnMut(&str)) {
    if attr.path.segments.is_empty() {
        return;
    }

    let id = &attr.path.segments[0].ident;
    if id.to_string() != "grapl" {
        return;
    }

    let parsed_attr_meta = attr.parse_meta().expect("malformed args");

    if let Meta::List(attrs) = parsed_attr_meta {
        for arg in attrs.nested {
            if let NestedMeta::Meta(Meta::Path(arg)) = arg {
                let attr_ident = arg.segments[0].ident.to_string();

                (on)(attr_ident.as_str());
            }
        }
    }
}

fn set_timestamp_from_meta(field: &Field, prop_name: &str, time_prop: &mut Option<String>) {
    for attr in &field.attrs {
        on_grapl_attrs(&attr, |meta_attr| {
            if meta_attr == prop_name {
                if time_prop.is_some() {
                    panic!("Can not set {} property more than once", prop_name);
                }
                *time_prop = Some(field.ident.clone().unwrap().to_string());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
