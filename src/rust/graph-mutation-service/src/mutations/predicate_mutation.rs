use crate::mutations::{UpsertGenerator, QueryInput};
use crate::v1beta1::NodeProperty;
use crate::mutations::escape::{Escaped, escape_quote};
use crate::v1beta1::node_property;
use crate::mutations::immutable_string_mutation::ImmutableStringUpsertGenerator;
use grapl_graph_descriptions::ImmutableStrProp;
use crate::mutations::incr_only_uint_mutation::{IncrementOnlyUintUpsertGenerator};
use crate::mutations::immutable_int_mutation::ImmutableIntUpsertGenerator;
use crate::mutations::immutable_uint_mutation::ImmutableUintUpsertGenerator;
use crate::mutations::decr_only_uint_mutation::DecrementOnlyUintUpsertGenerator;
use crate::mutations::incr_only_int_mutation::IncrementOnlyIntUpsertGenerator;
use crate::mutations::decr_only_int_mutation::DecrementOnlyIntUpsertGenerator;

#[derive(Default)]
pub struct NodePropertyUpsertGenerator {
    immutable_str_upsert_generator: ImmutableStringUpsertGenerator,
    immutable_uint_upsert_generator: ImmutableUintUpsertGenerator,
    incr_uint_upsert_generator: IncrementOnlyUintUpsertGenerator,
    decr_uint_upsert_generator: DecrementOnlyUintUpsertGenerator,
    immutable_int_upsert_generator: ImmutableIntUpsertGenerator,
    incr_int_upsert_generator: IncrementOnlyIntUpsertGenerator,
    decr_int_upsert_generator: DecrementOnlyIntUpsertGenerator,
}

impl UpsertGenerator for NodePropertyUpsertGenerator {
    type Input = NodeProperty;
    fn generate_upserts(&mut self, creation_query: &QueryInput<'_>, predicate_name: &str, value: &Self::Input) -> (&str, &[dgraph_tonic::Mutation]) {
        match &value.property {
            Some(node_property::Property::ImmutableStr(prop)) => {
                self.immutable_str_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::ImmutableUint(prop)) => {
                self.immutable_uint_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::IncrementOnlyUint(prop)) => {
                self.incr_uint_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::DecrementOnlyUint(prop)) => {
                self.decr_uint_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::ImmutableInt(prop)) => {
                self.immutable_int_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::IncrementOnlyInt(prop)) => {
                self.incr_int_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            Some(node_property::Property::DecrementOnlyInt(prop)) => {
                self.decr_int_upsert_generator.generate_upserts(
                    creation_query,
                    predicate_name,
                    prop,
                )
            }
            None => panic!("Invalid property"),
        }
    }
}
