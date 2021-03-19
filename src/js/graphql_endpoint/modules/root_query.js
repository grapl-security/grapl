const {
	GraphQLObjectType,
	GraphQLInt,
	GraphQLString,
	GraphQLList,
	GraphQLSchema,
	GraphQLNonNull,
} = require("graphql");

const {
    LensNodeType
} = require("./schema");

const {
    getDgraphClient
} = require("./dgraph_client");


const getLenses = async (dg_client, first, offset) => {
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

const getLensSubgraphByName = async (dg_client, lens_name) => {
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


const filterDefaultDgraphNodeTypes = (node_type) => {
	return node_type !== "Base" && node_type !== "Entity";
};

const handleLensScope = async (parent, args) => {
	console.log("handleLensScope args: ", args);
	const dg_client = getDgraphClient();

	const lens_name = args.lens_name;
	console.log("lens_name in handleLensScope", lens_name);

	// grab the graph of lens, lens scope, and neighbors to nodes in-scope of the lens ((lens) -> (neighbor) -> (neighbor's neighbor))
	const lens_subgraph = await getLensSubgraphByName(dg_client, lens_name);
	console.log("lens_subgraph in handleLensScope: ", lens_subgraph);

	lens_subgraph["uid"] = parseInt(lens_subgraph["uid"], 16);
	// if it's undefined/null, might as well make it an array
	lens_subgraph["scope"] = lens_subgraph["scope"] || [];

	// start enriching the nodes within the scope
	lens_subgraph["scope"].forEach(
		(neighbor) => (neighbor["uid"] = parseInt(neighbor["uid"], 16))
	);
	lens_subgraph["scope"].forEach(
		(neighbor) =>
			(neighbor["dgraph_type"] = neighbor["dgraph_type"].filter(
				filterDefaultDgraphNodeTypes
			))
	);
	// No dgraph_type? Not a node; skip it!
	lens_subgraph["scope"] = lens_subgraph["scope"].filter(
		(neighbor) => neighbor["dgraph_type"].length > 0
	);

	// record the uids of all direct neighbors to the lens.
	// These are the only nodes we should keep by the end of this process.
	// We'll then try to get all neighbor connections that only correspond to these nodes
	const neighbor_uids = new Set(
		lens_subgraph["scope"].map((node) => node["uid"])
	);

	// lens neighbors
	for (let neighbor of lens_subgraph["scope"]) {
		// neighbor of a lens neighbor
		for (const predicate in neighbor) {
			// we want to keep risks and enrich them at the same time
			if (predicate === "risks") {
				neighbor[predicate].forEach((risk_node) => {
					risk_node["uid"] = parseInt(risk_node["uid"], 16);
					
					if ("dgraph_type" in risk_node) {
						console.log("checking if dgraph_type in risk_node", risk_node);
						risk_node["dgraph_type"] = risk_node["dgraph_type"].filter(
							filterDefaultDgraphNodeTypes
						);
					}
				});

				// filter out nodes that don't have dgraph_types
				neighbor[predicate] = neighbor[predicate].filter(
					(node) => "dgraph_type" in node && !!node["dgraph_type"]
				);
				continue;
			}

			// If this edge is 1-to-many, we need to filter down the list to lens-neighbor -> lens-neighbor connections
			if (
				Array.isArray(neighbor[predicate]) &&
				neighbor[predicate] &&
				neighbor[predicate][0]["uid"]
			) {
				neighbor[predicate].forEach(
					(node) => (node["uid"] = parseInt(node["uid"], 16))
				);
				neighbor[predicate].forEach(
					(node) =>
						(node["dgraph_type"] = node["dgraph_type"].filter(
							filterDefaultDgraphNodeTypes
						))
				);
				neighbor[predicate] = neighbor[predicate].filter((second_neighbor) =>
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
					neighbor[predicate]["uid"] = parseInt(neighbor[predicate]["uid"], 16);
					neighbor[predicate]["dgraph_type"] = neighbor[predicate][
						"dgraph_type"
					].filter(filterDefaultDgraphNodeTypes);
				}
			}
		}
	}

	for (node of lens_subgraph["scope"]) {
		if (!builtins.has(node.dgraph_type[0])) {
			const tmpNode = { ...node };
			node.predicates = tmpNode;
		}
	}

	console.log("lens_subgraph scope", JSON.stringify(lens_subgraph["scope"]));
	return lens_subgraph;
};


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
			resolve: async (parent, args) => {
				console.log("lenses query arguments", args);
				const first = args.first;
				const offset = args.offset;
				// #TODO: Make sure to validate that 'first' is under a specific limit, maybe 1000
				console.log("Making getLensesQuery");
				const lenses = await getLenses(getDgraphClient(), first, offset);
				console.debug("returning data from getLenses for lenses resolver", lenses);
				return lenses;
			},
		},
		lens_scope: {
			type: LensNodeType,
			args: {
				lens_name: { type: new GraphQLNonNull(GraphQLString) },
			},
			resolve: async (parent, args) => {
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

module.exports = new GraphQLSchema({
	query: RootQuery,
});