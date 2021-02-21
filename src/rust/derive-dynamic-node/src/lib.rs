#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{parse_quote,
          Attribute,
          Data,
          Field,
          Fields,
          Ident,
          Meta,
          NestedMeta,
          Type};

const CREATE_TIME: &'static str = "create_time";
const LAST_SEEN_TIME: &'static str = "last_seen_time";
const TERMINATE_TIME: &'static str = "terminate_time";
const IMMUTABLE: &'static str = "immutable";
const INCREMENT: &'static str = "increment";
const DECREMENT: &'static str = "decrement";

fn name_and_ty(field: &Field) -> (&Ident, &Type, String) {
    let mut resolution = None;
    for attr in &field.attrs {
        on_grapl_attrs(&attr, |attr| {
            match attr {
                IMMUTABLE => resolution = Some(attr.to_string()),
                INCREMENT => resolution = Some(attr.to_string()),
                DECREMENT => resolution = Some(attr.to_string()),
                _ => (),
            };
        });
    }
    let property_name = field.ident.as_ref().unwrap();
    let resolution = resolution.unwrap_or_else(|| {
        panic!("property {} must have resolution set", property_name);
    });

    (property_name, &field.ty, resolution)
}

#[proc_macro_derive(NodeDescription)]
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
        .map(name_and_ty)
        .map(|(name, ty, resolution)| property_methods(name, ty, resolution))
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

    let use_the_dead_code_method_name = format!("__avoid_dead_code_lint{}", struct_name);
    let use_the_dead_code_method_name =
        syn::Ident::new(&use_the_dead_code_method_name, struct_name.span());

    let mut avoid_dead_code = quote!();
    for field in fields {
        let property_name = field.ident.as_ref().unwrap();
        let use_it = quote!(let _ = &self . #property_name;);
        avoid_dead_code.extend(use_it);
    }

    let q = quote!(

        impl #struct_name {
            #[allow(warnings)]
            fn #use_the_dead_code_method_name(&self) {
                unreachable!();
                #avoid_dead_code
            }
        }


        #[derive(Clone, Debug)]
        pub struct #node_name {
            dynamic_node: grapl_graph_descriptions::graph_description::NodeDescription,
        }

        pub trait #node_trait_name {
            fn get_mut_dynamic_node(&mut self) -> &mut grapl_graph_descriptions::graph_description::NodeDescription;
            fn get_dynamic_node(&self) -> &grapl_graph_descriptions::graph_description::NodeDescription;

            #methods
        }

        impl #node_name {
            pub fn new(strategy: grapl_graph_descriptions::graph_description::IdStrategy) -> Self {
                let mut properties = std::collections::HashMap::with_capacity(1);

                let dynamic_node = grapl_graph_descriptions::graph_description::NodeDescription {
                    node_type: #struct_name_string .to_owned(),
                    id_strategy: vec![strategy],
                    node_key: uuid::Uuid::new_v4().to_string(),
                    properties,
                };

                Self { dynamic_node }
            }

            pub fn get_node_key(&self) -> &str {
                &self.dynamic_node.node_key
            }

            pub fn clone_node_key(&self) -> String {
                self.dynamic_node.clone_node_key()
            }

            pub fn into_dyn_node(self) -> NodeDescription {
                self.dynamic_node
            }
        }

        impl AsRef<grapl_graph_descriptions::graph_description::NodeDescription> for #node_name {
            fn as_ref(&self) -> &NodeDescription {
                &self.dynamic_node
            }
        }

        impl AsMut<grapl_graph_descriptions::graph_description::NodeDescription> for #node_name {
            fn as_mut(&mut self) -> &mut NodeDescription {
                &mut self.dynamic_node
            }
        }

        impl Into<grapl_graph_descriptions::graph_description::NodeDescription> for #node_name {
            fn into(self) -> NodeDescription {
                self.dynamic_node
            }
        }
    );

    q.into()
}

// fn get_static_identifiers()

#[proc_macro_derive(GraplStaticId, attributes(grapl))]
pub fn derive_grapl_static(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_struct = match input.data {
        Data::Struct(input_struct) => input_struct,
        _ => panic!("Only available for struct"),
    };

    let fields = match input_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Requires named fields"),
    };

    let mut id_fields = quote!();
    fields.iter().for_each(|field| {
        for attr in &field.attrs {
            on_grapl_attrs(attr, |meta_attr| {
                if meta_attr == "static_id" {
                    let f = field
                        .ident
                        .as_ref()
                        .expect("field is missing an identifier")
                        .to_string();
                    id_fields.extend(quote!(#f .to_string(), ));
                }
            });
        }
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

            pub fn identity_strategy() -> IdStrategy {
                return #node_name :: static_strategy()
            }
        }
    );

    q.into()
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

    let mut create_time_prop: Option<String> = None;
    let mut last_seen_time_prop: Option<String> = None;
    let mut terminate_time_prop: Option<String> = None;

    for field in fields.iter() {
        assert_meta_attr_combo(field, CREATE_TIME, IMMUTABLE);
        assert_meta_attr_combo(field, TERMINATE_TIME, IMMUTABLE);
        set_timestamp_from_meta(field, CREATE_TIME, &mut create_time_prop);
        set_timestamp_from_meta(field, LAST_SEEN_TIME, &mut last_seen_time_prop);
        set_timestamp_from_meta(field, TERMINATE_TIME, &mut terminate_time_prop);
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
    let create_time_prop =
        create_time_prop.expect("missing property with attribute: create_time_prop");
    let last_seen_time_prop =
        last_seen_time_prop.expect("missing property with attribute: last_seen_time_prop");
    let terminate_time_prop =
        terminate_time_prop.expect("missing property with attribute: terminated_time_prop");

    let node_name_str = format!("{}Node", struct_name);
    let node_name = syn::Ident::new(&node_name_str, struct_name.span());
    // Add node name to id
    let q = quote!(
        impl #node_name {
            pub fn session_strategy() -> IdStrategy {
                Session {
                    created_time: #create_time_prop . to_string(),
                    last_seen_time: #last_seen_time_prop . to_string(),
                    terminated_time: #terminate_time_prop . to_string(),
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

fn assert_meta_attr_combo(field: &Field, meta_attr_match_a: &str, meta_attr_match_b: &str) {
    let mut a_matched = false;
    let mut b_matched = false;
    for attr in &field.attrs {
        on_grapl_attrs(&attr, |meta_attr| {
            a_matched |= meta_attr == meta_attr_match_a;
            b_matched |= meta_attr == meta_attr_match_b;
        });
    }
    if a_matched && !b_matched {
        panic!(
            "expected {} and {} to be true, got {}: {} and {}: {}",
            meta_attr_match_a,
            meta_attr_match_b,
            meta_attr_match_a,
            a_matched,
            meta_attr_match_b,
            b_matched,
        )
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

fn resolvable_type_from(
    property_type: &Type,
    resolution_name: &str,
) -> Option<(syn::Type, syn::Ident)> {
    let (return_type, method_ident): (syn::Type, syn::Ident) = match property_type {
        // janky way to get String="fully::qualified::path::Type" given a TypePath
        Type::Path(typepath) => {
            let typepath = typepath
                .path
                .segments
                .iter()
                .into_iter()
                .map(|x| x.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");
            match (typepath.as_ref(), resolution_name) {
                /* underlying struct field type    maps to this type   via this method on NodeProperty */
                ("String", IMMUTABLE) => (
                    parse_quote!(grapl_graph_descriptions::ImmutableStrProp),
                    parse_quote!(as_immutable_str),
                ),
                ("std::string::String", IMMUTABLE) => (
                    parse_quote!(grapl_graph_descriptions::ImmutableStrProp),
                    parse_quote!(as_immutable_str),
                ),
                ("u64", IMMUTABLE) => (
                    parse_quote!(grapl_graph_descriptions::ImmutableUintProp),
                    parse_quote!(as_immutable_uint),
                ),
                ("u64", INCREMENT) => (
                    parse_quote!(grapl_graph_descriptions::IncrementOnlyUintProp),
                    parse_quote!(as_increment_only_uint),
                ),
                ("u64", DECREMENT) => (
                    parse_quote!(grapl_graph_descriptions::DecrementOnlyUintProp),
                    parse_quote!(as_decrement_only_uint),
                ),
                ("i64", IMMUTABLE) => (
                    parse_quote!(grapl_graph_descriptions::ImmutableIntProp),
                    parse_quote!(as_immutable_int),
                ),
                ("i64", INCREMENT) => (
                    parse_quote!(grapl_graph_descriptions::IncrementOnlyIntProp),
                    parse_quote!(as_increment_only_int),
                ),
                ("i64", DECREMENT) => (
                    parse_quote!(grapl_graph_descriptions::DecrementOnlyIntProp),
                    parse_quote!(as_decrement_only_int),
                ),
                _ => return None,
            }
        }
        // If you're seeing this panic, then a field on the struct you're deriving
        // doesn't resolve to a TypePath.  That's a corner case, and assuming
        // you don't actually need a getter for it, it can be handled explicitly
        // with a no-op matcher.
        _ => panic!("Tried to dynamically construct getter for unrecognized type!"),
    };

    Some((return_type, method_ident))
}

fn property_methods(property_name: &Ident, property_type: &Type, resolution_name: String) -> TS2 {
    let get_method_name = format!("get_{}", property_name);
    let get_method_name = syn::Ident::new(&get_method_name, property_name.span());

    let with_method_name = format!("with_{}", property_name);
    let with_method_name = syn::Ident::new(&with_method_name, property_name.span());

    let property_name_str = format!("{}", property_name);

    let mut implementation: TS2 = quote!();

    let (return_type, method_ident) = match resolvable_type_from(&property_type, &resolution_name) {
        Some(property_type) => property_type,
        None => return implementation,
    };

    let with_method_implementation = quote!(
        fn #with_method_name(&mut self, #property_name: impl Into<#return_type>) -> &mut Self {
            let #property_name = #property_name .into();
            self.get_mut_dynamic_node()
            .properties.insert(
                #property_name_str .to_string(),
                #property_name .into(),
            );
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

    let get_method_implementation = quote!(
        fn #get_method_name(&self) -> Option<#return_type> {
            let property_result: Option<&NodeProperty> = self.get_dynamic_node().get_property(#property_name_str);

            let property_result = match property_result {
                Some(property_result) => property_result. #method_ident(),
                None => return None
            };
            match property_result {
              Some(property) => Some(#return_type (property.to_owned())),
              None => None,
            }
        }
    );
    implementation.extend(get_method_implementation);

    implementation
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
