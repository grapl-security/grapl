import {
	GraphQLObjectType,
	GraphQLInt,
	GraphQLString,
	GraphQLList,
	GraphQLSchema,
	GraphQLNonNull,
} from "graphql";

import {
	LensNodeType,
	builtins,
} from "./schema";

import {
	getDgraphClient,
	DgraphClient,
	RawNode,
} from "./dgraph_client";

type MysteryParentType = never;

const getLenses = async (dg_client: DgraphClient, first: number, offset: number) => {
	console.log("first, offset parameters in getLenses()", first, offset);

	const query = `
		query all($a: int, $b: int)
		{
			all(func: type(Lens), first: $a, offset: $b, orderdesc: score)
			{
				lens_name,
				score,
				node_key,
				uid,
				dgraph_type: dgraph.type,
				lens_type,
				scope {
					uid,
					node_key,
					dgraph_type: dgraph.type,
				}
			}
		}
	`;

	console.log("Creating DGraph txn in getLenses");

	const txn = dg_client.newTxn();

	try {
		console.log("Querying DGraph for lenses in getLenses");
		const res = await txn.queryWithVars(query, {
			$a: first.toString(),
			$b: offset.toString(),
		});
		console.log("Lens response from DGraph", res);
		return res.getJson()["all"];
	} catch (e) {
		console.error("Error in DGraph txn getLenses: ", e);
	} finally {
		console.log("Closing Dgraph Txn in getLenses")
		await txn.discard();
	}
};

const getLensSubgraphByName = async (dg_client: DgraphClient, lens_name: string) => {
	const query = `
		query all($a: string, $b: first, $c: offset) {
			all(func: eq(lens_name, $a), first: 1) {
				uid,
				dgraph_type: dgraph.type,
				node_key,
				lens_name,
				lens_type,
				score,
				scope @filter(has(node_key)) {
					uid,
					dgraph_type: dgraph.type,
					expand(_all_) {
						uid,
						dgraph_type: dgraph.type,
						expand(_all_)
					}
				}
			}
		}
    `;

	console.log("Creating DGraph txn in getLensSubgraphByName");
	const txn = dg_client.newTxn();

	try {
		console.log("Querying DGraph in getLensSubgraphByName");
		const res = await txn.queryWithVars(query, { $a: lens_name });
		console.log("returning following data from getLensSubGrapByName: ", res.getJson()["all"][0])
		return res.getJson()["all"][0];
	} catch (e) {
		console.error("Error in DGraph txn: getLensSubgraphByName", e);
	} finally {
		console.log("Closing dgraphtxn in getLensSubraphByName")
		await txn.discard();
	}
};


const filterDefaultDgraphNodeTypes = (node_type: string) => {
	return node_type !== "Base" && node_type !== "Entity";
};

function coerceUidIntoInt(node: RawNode) { 
	if(typeof node["uid"] == 'string') {
		node["uid"] = parseInt(node["uid"], 16);
	}
}

function enrichNode(node: RawNode) {
	coerceUidIntoInt(node);
	node["dgraph_type"] = node["dgraph_type"].filter(
		filterDefaultDgraphNodeTypes
	);
}

const AWS = require("aws-sdk");
const IS_LOCAL = process.env.IS_LOCAL == "True" || null;

const getDisplayProperty = async (nodeType: string) => {
	try {
		const region = process.env.AWS_REGION;
		AWS.config.update({ region: region });

		const ddb = new AWS.DynamoDB({
			// new client
			apiVersion: "2012-08-10",
			region: IS_LOCAL ? process.env.AWS_REGION : undefined,
			accessKeyId: IS_LOCAL ? process.env.DYNAMODB_ACCESS_KEY_ID : undefined,
			secretAccessKey: IS_LOCAL
				? process.env.DYNAMODB_ACCESS_KEY_SECRET
				: undefined,
			endpoint: IS_LOCAL ? process.env.DYNAMODB_ENDPOINT : undefined,
		});

		const params = {
			TableName: process.env.GRAPL_DISPLAY_TABLE,
			Key: {
				node_type: { S: nodeType }, // get display prop for a given node based on the type
			},
			ProjectionExpression: "display_property", // identifies	the attributes that you want to query for
		};

		const response = await ddb.getItem(params).promise();

		if (response.Item === undefined) {
			return "dgraph_type";
		}
		return response.Item.display_property;
	} catch (e) {
		console.error(
			"Error Querying DynamoDB for display property in root_query.ts",
			e
		);
	}
};

const handleLensScope = async (parent: MysteryParentType, args: LensArgs) => {
	console.log("handleLensScope args: ", args);
	const dg_client = getDgraphClient();

	const lens_name = args.lens_name;
	console.log("lens_name in handleLensScope", lens_name);

	// grab the graph of lens, lens scope, and neighbors to nodes in-scope of the lens ((lens) -> (neighbor) -> (neighbor's neighbor))
	const lens_subgraph = await getLensSubgraphByName(dg_client, lens_name);
	console.log("lens_subgraph in handleLensScope: ", lens_subgraph);

	coerceUidIntoInt(lens_subgraph);
	// if it's undefined/null, might as well make it an array
	lens_subgraph["scope"] = lens_subgraph["scope"] || [];

	// start enriching the nodes within the scope
	lens_subgraph["scope"].forEach(enrichNode);

	// No dgraph_type? Not a node; skip it!
	lens_subgraph["scope"] = lens_subgraph["scope"].filter(
		(neighbor: RawNode) => neighbor["dgraph_type"].length > 0
	);

	// record the uids of all direct neighbors to the lens.
	// These are the only nodes we should keep by the end of this process.
	// We'll then try to get all neighbor connections that only correspond to these nodes
	const neighbor_uids = new Set(
		lens_subgraph["scope"].map((node: RawNode) => node["uid"])
	);

	// lens neighbors
	for (let neighbor of lens_subgraph["scope"]) {
		// neighbor of a lens neighbor
		for (const predicate in neighbor) {
			// we want to keep risks and enrich them at the same time
			if (predicate === "risks") {
				neighbor[predicate].forEach((risk_node: RawNode) => {
					coerceUidIntoInt(risk_node);
					
					if ("dgraph_type" in risk_node) {
						console.log("checking if dgraph_type in risk_node", risk_node);
						risk_node["dgraph_type"] = risk_node["dgraph_type"].filter(
							filterDefaultDgraphNodeTypes
						);
					}
				});

				// filter out nodes that don't have dgraph_types
				neighbor[predicate] = neighbor[predicate].filter(
					(node: RawNode) => "dgraph_type" in node && !!node["dgraph_type"]
				);
				continue;
			}

			// If this edge is 1-to-many, we need to filter down the list to lens-neighbor -> lens-neighbor connections
			if (
				Array.isArray(neighbor[predicate]) &&
				neighbor[predicate] &&
				neighbor[predicate][0]["uid"]
			) {
				neighbor[predicate].forEach(enrichNode);
				neighbor[predicate] = neighbor[predicate].filter((second_neighbor: RawNode) =>
					neighbor_uids.has(second_neighbor["uid"])
				);

				// If we filtered all the edges down, might as well delete this predicate
				if (neighbor[predicate].length === 0) {
					delete neighbor[predicate];
				}
			}
			// If this edge is 1-to-1, we need to determine if we need to delete the edge
			else if (
				typeof neighbor[predicate] === "object" &&
				neighbor[predicate]["uid"]
			) {
				if (!neighbor_uids.has(parseInt(neighbor[predicate]["uid"], 16))) {
					delete neighbor[predicate];
				} else {
					enrichNode(neighbor[predicate]);
				}
			}
		}
	}

	for (const node of lens_subgraph["scope"]) {
		const nodeType = node.dgraph_type.filter(filterDefaultDgraphNodeTypes)[0];
		const displayProperty = await getDisplayProperty(nodeType);

		if (node[displayProperty.S] === undefined) {
			node["display"] = nodeType;
		} else {
			node["display"] = node[displayProperty.S].toString();
		}
	}

	for (const node of lens_subgraph["scope"]) {
		if (!builtins.has(node.dgraph_type[0])) {
			const tmpNode = { ...node };
			node.predicates = tmpNode;
		}
	}

	console.log("lens_subgraph scope", JSON.stringify(lens_subgraph["scope"]));
	return lens_subgraph;
};

interface RootQueryArgs {
	first: number;
	offset: number;
}

interface LensArgs {
	lens_name: string;
}

const RootQuery = new GraphQLObjectType({
	name: "RootQueryType",
	fields: {
		lenses: {
			type: GraphQLList(LensNodeType),
			args: {
				first: {
					type: new GraphQLNonNull(GraphQLInt),
				},
				offset: {
					type: new GraphQLNonNull(GraphQLInt),
				},
			},
			resolve: async (parent: MysteryParentType, args: RootQueryArgs) => {
				console.log("lenses query arguments", args);
				const first = args.first;
				const offset = args.offset;
				// #TODO: Make sure to validate that 'first' is under a specific limit, maybe 1000
				console.log("Making getLensesQuery");
				const lenses = await getLenses(getDgraphClient(), first, offset);
				console.debug("eturning data from getLenses for lenses resolver", lenses);
				return lenses;
			},
		},
		lens_scope: {
			type: LensNodeType,
			args: {
				lens_name: { type: new GraphQLNonNull(GraphQLString) },
			},
			resolve: async (parent: MysteryParentType, args: LensArgs) => {
				try {
					console.log("lens_scope args: ", args);
					let response = await handleLensScope(parent, args);
					console.log("lens_scope response: ", response);
					return response;
				} catch (e) {
					console.error("Error in handleLensScope: ", e);
					throw e;
				}
			},
		},
	},
});

export const RootQuerySchema = new GraphQLSchema({ 
	query: RootQuery,
});