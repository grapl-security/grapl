const dgraph = require("dgraph-js");
const grpc = require("grpc");
const { GraphQLJSONObject } = require('graphql-type-json');

const { 
    GraphQLObjectType, 
    GraphQLInt, 
    GraphQLString, 
    GraphQLList, 
    GraphQLSchema, 
    GraphQLUnionType, 
    GraphQLNonNull
}  = require('graphql');

const { GraphQLBoolean } = require("graphql");

const BaseNode = {
    uid: {type: GraphQLInt},
    node_key: {type: GraphQLString}, 
    dgraph_type: {type: GraphQLList(GraphQLString)},
}

const LensNodeType = new GraphQLObjectType({
    name: "LensNode", 
    fields: () => ({
        ...BaseNode,
        lens_name: {type: GraphQLString}, 
        score: {type: GraphQLInt}, 
        scope: {type: GraphQLList(GraplEntityType)},
        lens_type: {type: GraphQLString}, 
    })
})

const RiskType = new GraphQLObjectType({
    name: 'Risk',
    fields: {
        ...BaseNode,
        analyzer_name: {type: GraphQLString}, 
        risk_score: {type: GraphQLInt},
    }
})

// We have to support every type in grapl_analyzerlib/schemas
// We also have to support dynamic types, which would map to plugins,
// probably using the GraphQLJsonType

// TODO: File is missing all of its properties
const FileType = new GraphQLObjectType({
    name : 'File',
    fields : {
        ...BaseNode,
        file_name: {type: GraphQLString},
        file_type: {type: GraphQLString},
        file_extension: {type: GraphQLString},
        file_mime_type: {type: GraphQLString},
        file_size: {type: GraphQLInt},
        file_version: {type: GraphQLString}, 
        file_description: {type: GraphQLString},
        file_product: {type: GraphQLString},
        file_company: {type: GraphQLString}, 
        file_directory: {type: GraphQLString},
        file_inode: {type: GraphQLInt},
        file_hard_links: {type: GraphQLString}, 
        signed: {type: GraphQLBoolean},
        signed_status: {type: GraphQLString}, 
        md5_hash: {type: GraphQLString},
        sha1_hash: {type: GraphQLString},
        sha256_hash: {type: GraphQLString},
        risks: {type: GraphQLList(RiskType)},
        file_path: {type: GraphQLString},
    }
});

const IpConnections = new GraphQLObjectType({
    name: 'IpConnections',
    fields: () => ({
        ...BaseNode,
        risks: {type: GraphQLList(RiskType)},
        src_ip_addr: {type: GraphQLString},
        src_port: {type: GraphQLString},
        dst_ip_addr: {type: GraphQLString},
        dst_port: {type: GraphQLString},
        created_timestamp: {type: GraphQLInt},
        terminated_timestamp: {type: GraphQLInt},
        last_seen_timestamp: {type: GraphQLInt},
        inbound_ip_connection_to: {type: IpAddressType},
    })
})

// TODO: Process is missing many properties and edges
// 'fields' is a callback, so that we can declare ProcessType first, and then
// reference it in 'children' later
// This is called lazy evaluation, where we defer the execution of code until it is needed
const ProcessType = new GraphQLObjectType({
    name : 'Process',
    fields : () => ({
        ...BaseNode,
        created_timestamp: {type: GraphQLInt},
        image_name: {type: GraphQLString},
        process_name: {type: GraphQLString},
        arguments: {type: GraphQLString}, 
        children: {
            type: GraphQLList(ProcessType) 
        },
        bin_file: {type: FileType},
        created_file: {type: FileType},
        deleted_files: {type:FileType},
        read_files: {type: GraphQLList(FileType)},
        wrote_files: {type: GraphQLList(FileType)},
        created_connections: {type: GraphQLList(ProcessOutboundConnections)},
        inbound_connections: {type: GraphQLList(ProcessInboundConnections)},
        process_id: {type: GraphQLInt},
        risks: {type: GraphQLList(RiskType)},
    })
});
const NetworkConnection = new GraphQLObjectType({
    name: 'NetworkConnection',
    fields: () => ({
        src_ip_address: {type: GraphQLString}, 
        src_port: {type: GraphQLString}, 
        dst_ip_address: {type: GraphQLString}, 
        dst_port: {type: GraphQLString}, 
        created_timestamp: {type: GraphQLInt}, 
        terminated_timestamp: {type: GraphQLInt},
        last_seen_timestamp: {type: GraphQLInt},
        inbound_network_connection_to: {type: GraphQLList(IpPort)},
    })
}) 

const IpPort = new GraphQLObjectType({
    name: 'IpPort',
    fields: {
        ...BaseNode,
        ip_address: {type: GraphQLString},
        protocol: {type: GraphQLString},
        port: {type: GraphQLInt}, 
        first_seen_timestamp: {type: GraphQLInt}, 
        last_seen_timestamp: {type: GraphQLInt}, 
        network_connections: {type: GraphQLList(NetworkConnection)},
    }
})

const IpAddressType = new GraphQLObjectType({
    name : 'IpAddress',
    fields : {
        ...BaseNode,
        risks: {type: GraphQLList(RiskType)},
        ip_address: {type: GraphQLString}
    }
});

const AssetType = new GraphQLObjectType({
    name : 'Asset',
    fields : {
        ...BaseNode,
        risks: {type: GraphQLList(RiskType)},
        hostname: {type: GraphQLString},
        asset_ip: {type: GraphQLList(IpAddressType)},
        asset_processes: {type: GraphQLList(ProcessType)}, 
        files_on_asset: {type: GraphQLList(FileType)},
    }
});


const ProcessInboundConnections = new GraphQLObjectType ({
    name: 'ProcessInboundConnections',
    fields: {
        ...BaseNode,
        ip_address: {type: GraphQLString},
        protocol: {type: GraphQLString}, 
        created_timestamp: {type: GraphQLInt}, 
        terminated_timestamp: {type: GraphQLInt},
        last_seen_timestamp: {type: GraphQLInt},
        port: {type: GraphQLInt},
        bound_port: {type: GraphQLList(IpPort)},
        bound_ip: {type: GraphQLList(IpAddressType)},
    }
})

const ProcessOutboundConnections = new GraphQLObjectType ({
    name: 'ProcessOutboundConnections',
    fields: {
        ...BaseNode,
        ip_address: {type: GraphQLString},
        protocol: {type: GraphQLString},
        created_timestamp: {type: GraphQLInt}, 
        terminated_timestamp: {type: GraphQLInt},
        last_seen_timestamp: {type: GraphQLInt},
        port: {type: GraphQLInt},
        connected_over: {type: GraphQLList(IpPort)},
        connected_to: {type: GraphQLList(IpPort)},
    }
})

const PluginType = new GraphQLObjectType({
    name: 'PluginType',
    fields: {
        predicates: { type: GraphQLJSONObject },
    }
})


const builtins = new Set([
    'Process',
    'File',
    'IpAddress',
    'Asset',
    'Risk',
    'IpConnections',
    'ProcessInboundConnections',
    'ProcessOutboundConnections',
])

// TODO: Handle the rest of the builtin types
const resolveType = (data) => {
    if (data.dgraph_type[0] === 'Process') {
        return 'Process';
    }

    if (data.dgraph_type[0] === 'File') {
        return 'File';
    }

    if (data.dgraph_type[0] === 'IpAddress') {
        return 'IpAddress';
    }
    
    if (data.dgraph_type[0] === 'Asset') {
        return 'Asset';
    }

    if (data.dgraph_type[0] === 'Risk'){
        return 'Risk';
    }

    if (data.dgraph_type[0] === 'IpConnections'){
        return 'IpConnections';
    }

    if (data.dgraph_type[0] === 'ProcessInboundConnections'){
        return 'ProcessInboundConnections';
    }

    if (data.dgraph_type[0] === 'ProcessOutboundConnections'){
        return 'ProcessOutboundConnections';
    }
    
    return 'PluginType'
};

// | FileType, ProcessType, IpAddressType, AssetType, RiskType, IpConnections, ProcessInboundConnections, ProcessOutboundConnections
const GraplEntityType = new GraphQLUnionType({
    name: 'GraplEntityType',
    types: [ PluginType, FileType, ProcessType, AssetType ],
    resolveType: resolveType
});

const getDgraphClient = () => {

    const clientStub = new dgraph.DgraphClientStub(
        // addr: optional, default: "localhost:9080"
        "master_graph:9080",
        // credentials: optional, default: grpc.credentials.createInsecure()
        grpc.credentials.createInsecure(),
    );

    return new dgraph.DgraphClient(clientStub);
}
// return lens
const getLenses = async (dg_client) => {
    const query = `
    {
        all(func: type(Lens))
        {
            lens_name: lens,
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
    }`;

    const txn = dg_client.newTxn();
    try {
        const res = await txn.query(query);
        return res.getJson()['all'];
    } finally {
        await txn.discard();
    }
}

// return lens
const getLensByName = async (dg_client, lensName) => {
    const query = `
    query all($a: string)
        {
            all(func: eq(lens, $a), first: 1)
            {
                lens_name: lens,
                score,
                node_key,
                uid,
                dgraph_type: dgraph.type,
                lens_type,
                scope {
                    uid,
                    dgraph_type: dgraph.type,
                    expand(_all_)
                }
            }
        }
    `;
    const txn = dg_client.newTxn();
    try {
        const res = await txn.queryWithVars(query, {'$a': lensName});
        return res.getJson()['all'][0];
    } finally {
        await txn.discard();
    }
}
// Return base node 
const getNeighborsFromNode = async (dg_client, nodeUid) => {
    const query = `
    query all($a: string)
    {
        all(func: uid($a), first: 1)
        {
            expand(_all_) {
                uid,
                dgraph_type: dgraph.type,
                expand(_all_)
            }
        }
    }`;
    const txn = dg_client.newTxn();
    try {
        const res = await txn.queryWithVars(query, {'$a': nodeUid});
        return res.getJson()['all'][0];
    } finally {
        await txn.discard();
    }
}

const getRisksFromNode = async (dg_client, nodeUid) => {
    if (!nodeUid) {
        console.warn('nodeUid can not be null, undefined, or empty')
        return
    }
    const query = `
    query all($a: string)
    {
        all(func: uid($a))
        {
            uid,
            risks {
                uid,
                dgraph_type: dgraph.type
                expand(_all_),
            }
        }
    }`;
    const txn = dg_client.newTxn();
    try {
        const res = await txn.queryWithVars(query, {'$a': nodeUid});
        return res.getJson()['all'][0]['risks'];
    } finally {
        await txn.discard();
    }
}


const inLensScope = async (dg_client, nodeUid, lensUid) => {

    const query = `
    query all($a: string, $b: string)
    {
        all(func: uid($b)) @cascade
        {
            uid,
            scope @filter(uid($a)) {
                uid,
            }
        }
    }`;

    const txn = dg_client.newTxn();
    try {
        const res = await txn.queryWithVars(query, {
            '$a': nodeUid, '$b': lensUid
        });
        const json_res = res.getJson();
        return json_res['all'].length !== 0;
    } finally {
        await txn.discard();
    }
}

const RootQuery = new GraphQLObjectType({
    name: 'RootQueryType', 
    fields: {
        lenses: {
            type: GraphQLList(LensNodeType),
            resolve: async (parent, args) => {
                const lenses = await getLenses(await getDgraphClient());
                return lenses;
            }
        },
        lens_scope:{
            type: LensNodeType, 
            args: {
                lens_name: {type: new GraphQLNonNull(GraphQLString)}
            },
            resolve: async (parent, args) => {
                const dg_client = getDgraphClient();

                const lens_name = args.lens_name;

                const lens = await getLensByName(dg_client, lens_name);
                for (const node of lens["scope"]) {
                    // node.uid = parseInt(node.uid, 16);
                    // for every node in our lens scope, get its neighbors

                    const nodeEdges = await getNeighborsFromNode(dg_client, node["uid"]);

                    for (const maybeNeighborProp in nodeEdges) {
                        const maybeNeighbor = nodeEdges[maybeNeighborProp];
                        // maybeNeighbor.uid = parseInt(maybeNeighbor.uid, 16);
                        
                        // A neighbor is either an array of objects with uid fields
                        if (Array.isArray(maybeNeighbor) && maybeNeighbor && maybeNeighbor[0].uid) {
                            const neighbors = maybeNeighbor;

                            for (const neighbor of neighbors) {
                                const isInScope = await inLensScope(dg_client, neighbor["uid"], lens["uid"]);
                                neighbor.uid = parseInt(neighbor.uid, 16);
                                if (isInScope) {
                                    if (Array.isArray(node[maybeNeighborProp])) {
                                        node[maybeNeighborProp].push(neighbor);
                                    } else {
                                        node[maybeNeighborProp] = [neighbor];
                                    }
                                }
                            }
                        }
                        else if (typeof maybeNeighbor === 'object' && maybeNeighbor.uid) {
                            const neighbor = maybeNeighbor;

                            const isInScope = await inLensScope(dg_client, neighbor["uid"], lens["uid"]);
                            neighbor.uid = parseInt(neighbor.uid, 16);
                            if (isInScope) {
                                if(!builtins.has(neighbor.dgraph_type[0])) {
                                    const tmpNode = {...neighbor};
                                    // Object.keys(node).forEach(function(key) { delete node[key]; });

                                    neighbor.predicates = tmpNode;
                                }
                                node[maybeNeighborProp] = neighbor
                            }
                        }
                    }

                }

                for (const node of lens["scope"]) {
                    try {
                        let nodeUid = node['uid'];
                        if (typeof nodeUid === 'number') {
                            nodeUid = '0x' + nodeUid.toString(16)
                        }
                        const risks = await getRisksFromNode(dg_client, nodeUid);
                        if (risks) {
                            for (const risk of risks) {
                                risk['uid'] = parseInt(risk['uid'], 16)
                            }
                            node['risks'] = risks;
                        }
                    } catch (err) {
                        console.error('Failed to get risks', err);
                    }
                    node.uid = parseInt(node.uid, 16);
                    // If it's a plugin we want to store the properties in a wrapper
                    if(!builtins.has(node.dgraph_type[0])) {
                        const tmpNode = {...node};
                        // Object.keys(node).forEach(function(key) {
                        //     if (Object.prototype.hasOwnProperty.call(node, key)) {
                        //         delete node[key];
                        //     }
                        // });

                        node.predicates = tmpNode;
                    }
                }


                lens.uid = parseInt(lens.uid, 16);
                return lens

            }
        }, 
        
    }
})


module.exports = new GraphQLSchema({
    query: RootQuery
});
