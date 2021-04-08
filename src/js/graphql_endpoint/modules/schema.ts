import {
  GraphQLObjectType,
  GraphQLInt,
  GraphQLString,
  GraphQLList,
} from "graphql";

export const BaseNode = {
  uid: { type: GraphQLInt },
  node_key: { type: GraphQLString },
  dgraph_type: { type: GraphQLList(GraphQLString) },
  display: { type: GraphQLString },
};

export const RiskType = new GraphQLObjectType({
  name: "Risk",
  fields: {
    ...BaseNode,
    analyzer_name: { type: GraphQLString },
    risk_score: { type: GraphQLInt },
  },
});
