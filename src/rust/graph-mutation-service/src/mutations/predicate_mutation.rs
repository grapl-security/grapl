use crate::mutations::{UpsertGenerator, QueryInput};
use crate::v1beta1::NodeProperty;
use crate::mutations::escape::{Escaped, escape_quote};
use crate::v1beta1::node_property;
use crate::mutations::immutable_string_mutation::ImmutableStringUpsertGenerator;
use grapl_graph_descriptions::ImmutableStrProp;
use crate::mutations::incr_only_uint_mutation::{IncrementOnlyUintUpsertGenerator};
use crate::mutations::immutable_int_mutation::ImmutableIntUpsertGenerator;
use crate::mutations::immutable_uint_mutation::ImmutableUintUpsertGenerator;

#[derive(Default)]
pub struct NodePropertyUpsertGenerator {
    immutable_str_upsert_generator: ImmutableStringUpsertGenerator,
    immutable_uint_upsert_generator: ImmutableUintUpsertGenerator,
    immutable_int_upsert_generator: ImmutableIntUpsertGenerator,
    incr_uint_upsert_generator: IncrementOnlyUintUpsertGenerator,
}

impl UpsertGenerator for NodePropertyUpsertGenerator {
    type Input = NodeProperty;
    fn generate_upserts(&mut self, creation_query: &QueryInput<'_>, predicate_name: &str, value: &Self::Input) -> (&str, &[dgraph_tonic::Mutation]) {
        match &value.property {
            Some(node_property::Property::IncrementOnlyUint(prop)) => {
                self.incr_uint_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::ImmutableUint(prop)) => {
                unimplemented!()
            }
            Some(node_property::Property::DecrementOnlyUint(prop)) => {
                unimplemented!()
            }
            Some(node_property::Property::DecrementOnlyInt(prop)) => {
                unimplemented!()
            }
            Some(node_property::Property::IncrementOnlyInt(prop)) => {
                unimplemented!()
            }
            Some(node_property::Property::ImmutableInt(prop)) => {
                unimplemented!()
            }
            Some(node_property::Property::ImmutableStr(prop)) => {
                self.immutable_str_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            None => panic!("Invalid property"),
        }
    }
}
