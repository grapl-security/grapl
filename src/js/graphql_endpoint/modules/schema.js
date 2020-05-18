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
// Lens Node Type
const LensNodeType = new GraphQLObjectType({
    name: "LensNode", 
    fields: () => ({
        ...BaseNode,
        lens_name: {type: GraphQLString}, 
        score: {type: GraphQLInt}, 
        scope: {type: GraphQLList(GraplEntityType)},
        
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
        process_id: {type: GraphQLInt},
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
    // If it is not a builtin, return JSON
    // return GraphQLJSONObject
};
const GraplEntityType = new GraphQLUnionType({
    name: 'GraplEntityType',
    types: [ PluginType | FileType, ProcessType, IpAddressType, AssetType, RiskType, IpConnections, ProcessInboundConnections, ProcessOutboundConnections ],
    resolveType: resolveType
});

const getDgraphClient = () => {

    const clientStub = new dgraph.DgraphClientStub(
        // addr: optional, default: "localhost:9080"
        "localhost:9081",
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
            scope {
                uid,
                node_key,
                dgraph_type: dgraph.type,
            }
        }
    }`;

    const res = await dg_client.newTxn().query(query);
    return res.getJson()['all'];
}

// return lens
const getLensByName = async (dg_client, lensName) => {
    console.log('lensname', lensName);
    const query = `
    query all($a: string)
        {
            all(func: eq(lens, $a), first: 1)
            {
                lens,
                score,
                node_key,
                uid,
                dgraph_type: dgraph.type,
                scope {
                    uid,
                    dgraph_type: dgraph.type,
                    expand(_all_)
                }
            }
        }
    `;

    const res = await dg_client.newTxn().queryWithVars(query, {'$a': lensName});
    return res.getJson()['all'][0];
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

    const res = await dg_client.newTxn().queryWithVars(query, {'$a': nodeUid});
    return res.getJson()['all'][0];    
}

const inLensScope = async (dg_client, nodeUid, lensUid) => {
    // console.log(nodeUid, lensUid);
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

    const res = await dg_client.newTxn().queryWithVars(query, {
        '$a': nodeUid, '$b': lensUid
    });
    return !!res.getJson()['all'];        
}

const RootQuery = new GraphQLObjectType({
    name: 'RootQueryType', 
    fields: {
        lenses: {
            type: GraphQLList(LensNodeType),
            resolve: async (parent, args) => {
                console.log("Getting Lenses")
                const lenses = await getLenses(await getDgraphClient());
                console.log("lenses", lenses);
                return lenses;
            }
        },
        lens_scope:{
            type: LensNodeType, 
            args: {
                lens_name: {type: new GraphQLNonNull(GraphQLString)}
            },
            resolve: async (parent, args) => {
                console.log('getLenseScope');
                const dg_client = getDgraphClient();

                const lens_name = args.lens_name;

                const lens = await getLensByName(dg_client, lens_name);
                for (const node of lens["scope"]) {
                    // for every node in our lens scope, get its neighbors

                    const nodeEdges = await getNeighborsFromNode(dg_client, node["uid"]);

                    for (const maybeNeighborProp in nodeEdges) {
                        const maybeNeighbor = nodeEdges[maybeNeighborProp];
                        
                        // A neighbor is either an array of objects with uid fields
                        if (Array.isArray(maybeNeighbor) && maybeNeighbor && maybeNeighbor[0].uid) {
                            const neighbors = maybeNeighbor;

                            for (const neighbor of neighbors) {
                                const isInScope = await inLensScope(dg_client, neighbor["uid"], lens["uid"]);
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
                                
                            if (isInScope) {
                                node[maybeNeighborProp] = neighbor
                            }                            
                        }
                    }

                    // If it's a plugin we want to store the properties in a wrapper
                    if(!builtins.has(node.dgraph_type[0])) {
                        const tmpNode = {...node};
                        Object.keys(node).forEach(function(key) { delete node[key]; });

                        node.properties = tmpNode;
                    }
                }
                console.log("Lens", lens)
                return lens

            }
        }, 
        
    }
})


module.exports = new GraphQLSchema({
    query: RootQuery
});