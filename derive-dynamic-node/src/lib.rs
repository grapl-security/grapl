#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Type};

/// #[derive(DynamicNode)]
/// pub struct Ec2Instance2 {
///     arn: String,
///     launch_time: u64
/// }

fn name_and_ty(field: &Field) -> (&Ident, &Type) {
    (field.ident.as_ref().unwrap(), &field.ty)
}

#[proc_macro_derive(DynamicNode)]
pub fn derive_dynamic_node(input: TokenStream) -> TokenStream {
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
        .map(|(name, ty)| get_method(name, ty))
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
            dynamic_node: graph_descriptions::graph_description::DynamicNode,
        }

        pub trait #node_trait_name {
            fn get_mut_dynamic_node(&mut self) -> &mut DynamicNode;

            #methods
        }

        impl #node_name {
            pub fn new(strategy: graph_descriptions::graph_description::IdStrategy, seen_at: u64) -> Self {
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

        impl AsRef<graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn as_ref(&self) -> &DynamicNode {
                &self.dynamic_node
            }
        }

        impl AsMut<graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn as_mut(&mut self) -> &mut DynamicNode {
                &mut self.dynamic_node
            }
        }

        impl Into<graph_descriptions::graph_description::DynamicNode> for #node_name {
            fn into(self) -> DynamicNode {
                self.dynamic_node
            }
        }

        impl Into<graph_descriptions::graph_description::Node> for #node_name {
            fn into(self) -> Node {
                self.dynamic_node.into()
            }
        }


        impl Into<graph_descriptions::graph_description::Node> for & #node_name {
            fn into(self) -> Node {
                self.dynamic_node.clone().into()
            }
        }


        impl Into<graph_descriptions::graph_description::Node> for &mut #node_name {
            fn into(self) -> Node {
                self.dynamic_node.clone().into()
            }
        }

    );

    q.into()

}


#[proc_macro_derive(GraplStaticId, attributes(grapl))]
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

    let id_fields = fields
        .iter()
        .filter_map(|field| {
            for attr in &field.attrs {
                if attr.path.segments.is_empty() {
                    return None
                }

                let id = &attr.path.segments[0].ident;
                if id.to_string() != "grapl" {
                    continue
                }

                return field.ident.as_ref()
            }

            None
        })
        .fold(quote!(), |mut acc, f| {
            let f = f.to_string();
            acc.extend(quote!(#f .to_string()));
            acc
        });

    assert!(id_fields.to_string().len() > 0);

    let struct_name = &input.ident;

    let node_name = format!("{}Node", struct_name);
    let node_name = syn::Ident::new(&node_name, struct_name.span());


    let strategy_name = format!("{}strategy", struct_name);
    let strategy_name = syn::Ident::new(&strategy_name, struct_name.span());

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


fn get_method(property_name: &Ident, property_type: &Type) -> TS2 {
    let method_name = format!("with_{}", property_name);
    let method_name = syn::Ident::new(&method_name, property_name.span());

    let property_name_str = format!("{}", property_name);
    quote!(
            fn #method_name(&mut self, #property_name: impl Into<#property_type>) -> &mut Self {
                let #property_name = #property_name .into();
                self.get_mut_dynamic_node()
                .properties.insert(
                    #property_name_str .to_string(),
                    #property_name .into(),
                );
                self
            }
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
