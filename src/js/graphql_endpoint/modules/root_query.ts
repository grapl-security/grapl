import { printSchema } from "graphql/utilities";
import {
    GraphQLObjectType,
    GraphQLInt,
    GraphQLString,
    GraphQLList,
    GraphQLSchema,
    GraphQLNonNull,
} from "graphql";

import { BaseNode, LensScopeQueryString } from "./schema";

import {
    getDgraphClient,
    DgraphClient,
    RawNode,
    EnrichedNode,
} from "./dgraph_client";
import { Schema, SchemaClient } from "./schema_client";
import { allSchemasToGraphql } from "./schema_to_graphql";
import { QueryGenerator } from "./query_generator";

type MysteryParentType = never;

const getLenses = async (
    dg_client: DgraphClient,
    first: number,
    offset: number
) => {
    console.debug("first, offset parameters in getLenses()", first, offset);

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

    console.debug("Creating DGraph txn in getLenses");

    const txn = dg_client.newTxn();

    try {
        console.debug("Querying DGraph for lenses in getLenses");
        const res = await txn.queryWithVars(query, {
            $a: first.toString(),
            $b: offset.toString(),
        });
        console.debug("Lens response from DGraph", res);
        return res.getJson()["all"];
    } catch (e) {
        console.error("Error in DGraph txn getLenses: ", e);
    } finally {
        console.debug("Closing Dgraph Txn in getLenses");
        await txn.discard();
    }
};

interface LensSubgraph {
    readonly node_key: string;
    readonly lens_name: string;
    readonly lens_type: string;
    readonly score: number;
    scope: RawNode[];
}

const getLensSubgraphByName = async (
    dg_client: DgraphClient,
    lens_name: string
) => {
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

    console.debug("Creating DGraph txn in getLensSubgraphByName");
    const txn = dg_client.newTxn();

    try {
        console.debug("Querying DGraph in getLensSubgraphByName");
        const res = await txn.queryWithVars(query, { $a: lens_name });
        console.debug(
            "returning following data from getLensSubGrapByName: ",
            res.getJson()["all"][0]
        );
        return res.getJson()["all"][0] as LensSubgraph & RawNode;
    } catch (e) {
        console.error("Error in DGraph txn: getLensSubgraphByName", e);
        throw e;
    } finally {
        console.debug("Closing dgraphtxn in getLensSubraphByName");
        await txn.discard();
    }
};

const filterDefaultDgraphNodeTypes = (node_type: string) => {
    return node_type !== "Base" && node_type !== "Entity";
};

function hasDgraphType(node: RawNode): boolean {
    return (
        "dgraph_type" in node && // it's a property
        !!node["dgraph_type"] && // it's not null
        node.dgraph_type?.length > 0
    );
}

function uidAsInt(node: RawNode): number {
    const uid = node["uid"];

    if (typeof uid == "string") {
        return parseInt(uid, 16);
    } else if (typeof uid == "number") {
        return uid;
    }
    throw new Error(`Oddly typed UID ${uid}`);
}

function asEnrichedNodeWithSchemas(
    node: RawNode,
    schemaMap: Map<string, Schema>
): EnrichedNode {
    const dgraph_types = (node.dgraph_type || []).filter(
        filterDefaultDgraphNodeTypes
    );
    const mostConcreteDgraphType = dgraph_types[0]; // yes, this can be undefined
    const whichPropToDisplay = schemaMap.get(dgraph_types[0])?.display_property;
    // fall back to just the type.
    // I don't super love this design - it's putting view logic in the controller layer
    const display: string =
        (node as any)[whichPropToDisplay] || mostConcreteDgraphType;
    return {
        ...node,
        uid: uidAsInt(node),
        dgraph_type: dgraph_types,
        display: display,
    };
}

const handleLensScope = async (
    parent: MysteryParentType,
    args: LensArgs,
    schemaMap: Map<string, Schema>
) => {
    console.debug("handleLensScope args: ", args);
    const dg_client = getDgraphClient();
    // partial apply schemaMap
    const asEnrichedNode = (node: RawNode) =>
        asEnrichedNodeWithSchemas(node, schemaMap);

    // Tosses ones that don't have dgraph type (i.e., edges)
    const batchEnrichNodes = (nodes: RawNode[]) => {
        return nodes.flatMap((n: RawNode) => {
            const enriched = asEnrichedNode(n);
            if (hasDgraphType(enriched)) {
                return [enriched];
            }
            return [];
        });
    };

    const lens_name = args.lens_name;

    // grab the graph of lens, lens scope, and neighbors to nodes in-scope of the lens ((lens) -> (neighbor) -> (neighbor's neighbor))
    const lens_subgraph: LensSubgraph & RawNode = await getLensSubgraphByName(
        dg_client,
        lens_name
    );
    console.debug("lens_subgraph in handleLensScope: ", lens_subgraph);

    lens_subgraph.uid = uidAsInt(lens_subgraph);
    let scope: EnrichedNode[] = batchEnrichNodes(lens_subgraph["scope"] || []);

    // record the uids of all direct neighbors to the lens.
    // These are the only nodes we should keep by the end of this process.
    // We'll then try to get all neighbor connections that only correspond to these nodes
    const uids_in_scope = new Set<number>(
        scope.map((node: EnrichedNode) => node.uid)
    );

    // lens neighbors
    for (const node of scope) {
        // neighbor of a lens neighbor
        for (const predicate in node) {
            // we want to keep risks and enrich them at the same time
            if (predicate === "risks") {
                // enrich + filter out nodes that don't have dgraph_types
                node[predicate] = batchEnrichNodes(node[predicate]);
                continue;
            }

            // If this edge is 1-to-many, we need to filter down the list to lens-neighbor -> lens-neighbor connections
            // i.e. if you see:
            //    Known UIDs: [1, 2, 3];
            //  (UID 1) --edge--> [2, 4, 5]
            // Throw away 4 and 5 since they're not in-scope.
            if (
                Array.isArray(node[predicate]) &&
                node[predicate] &&
                node[predicate][0]["uid"]
            ) {
                node[predicate] = batchEnrichNodes(node[predicate]).filter(
                    (neighbor_of_node: EnrichedNode) =>
                        uids_in_scope.has(neighbor_of_node["uid"])
                );

                // If we filtered all the edges down, might as well delete this predicate
                if (node[predicate].length === 0) {
                    delete node[predicate];
                }
            }
            // If this edge is 1-to-1, we need to determine if we need to delete the edge
            // i.e. if you see:
            //    Known UIDs: [1, 2, 3];
            //  (UID 1) --edge--> (UID 4)
            // Throw away this edge, since UID 4 is not in-scope.
            else if (
                typeof node[predicate] === "object" &&
                node[predicate]["uid"]
            ) {
                node[predicate] = asEnrichedNode(node[predicate]);
                if (!uids_in_scope.has(node[predicate].uid)) {
                    delete node[predicate];
                }
            }
        }
    }

    for (const node of scope) {
        if (!node) {
            throw new Error(
                `Somehow received a null or undefined scope node: ${node}`
            );
        }
    }

    lens_subgraph.scope = scope;
    console.debug(
        "lens_subgraph scope",
        JSON.stringify(lens_subgraph["scope"])
    );
    return lens_subgraph;
};

interface RootQueryArgs {
    readonly first: number;
    readonly offset: number;
}

interface LensArgs {
    readonly lens_name: string;
}

async function getRootQuery(): Promise<GraphQLObjectType> {
    const schemasWithBuiltins = await new SchemaClient().getSchemas();
    const schemas = schemasWithBuiltins.filter((schema) => {
        // This could be a one-liner, but I think it's complex enough for ifelse
        if (schema.node_type == "Risk" || schema.node_type == "Lens") {
            return false; // reject
        } else {
            return true; // keep
        }
    });

    // We use this in the `handleLensScope` to determine the display property
    const schemasMap: Map<string, Schema> = new Map(
        schemas.map((s) => [s.node_type, s])
    );

    const GraplEntityType = allSchemasToGraphql(schemas);
    const LensNodeType = new GraphQLObjectType({
        name: "LensNode",
        fields: () => ({
            ...BaseNode,
            lens_name: { type: GraphQLString },
            score: { type: GraphQLInt },
            scope: { type: GraphQLList(GraplEntityType) },
            lens_type: { type: GraphQLString },
        }),
    });

    return new GraphQLObjectType({
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
                resolve: async (
                    parent: MysteryParentType,
                    args: RootQueryArgs
                ) => {
                    console.debug("lenses query arguments", args);
                    const first = args.first;
                    const offset = args.offset;
                    // #TODO: Make sure to validate that 'first' is under a specific limit, maybe 1000
                    console.debug("Making getLensesQuery");
                    const lenses = await getLenses(
                        getDgraphClient(),
                        first,
                        offset
                    );
                    console.debug(
                        "returning data from getLenses for lenses resolver",
                        lenses
                    );
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
                        let response = await handleLensScope(
                            parent,
                            args,
                            schemasMap
                        );
                        return response;
                    } catch (e) {
                        console.error("Error in handleLensScope: ", e);
                        throw e;
                    }
                },
            },
            lens_scope_query: {
                type: LensScopeQueryString,
                resolve: async (parent: never, args: never) => {
                    // We should consider caching this instead of re-generating it every time.
                    const query_string = new QueryGenerator(
                        GraplEntityType
                    ).generate();
                    return {
                        query_string,
                    };
                },
            },
        },
    });
}

export async function getRootQuerySchema(): Promise<GraphQLSchema> {
    const schema = new GraphQLSchema({
        query: await getRootQuery(),
    });
    // Super useful for debugging!
    // console.log("Schema: ", printSchema(schema));
    return schema;
}
