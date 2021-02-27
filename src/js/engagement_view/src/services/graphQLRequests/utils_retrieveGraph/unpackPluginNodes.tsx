import {Node} from '../../../types/CustomTypes'
import {mapGraph} from "components/graphDisplay/graphLayout/graph_traverse";

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

export const unpackPluginNodes = (nodes: Node[]) => {
    for (const node of nodes) {
        if (!(node as any).predicates) {
            continue
        }
        mapGraph(node, (node, edge_name, neighbor) => {
            if ((node as any).predicates) {
                if(!builtins.has((node as any).predicates.dgraph_type[0])) {
                    // Using 'any' because the PluginType is temporary, and not valid outside of the initial response
                    const pluginNode = {...(node as any).predicates};
                    delete (node as any).predicates
                    Object.keys(pluginNode).forEach(function(key) { (node as any)[key] = pluginNode[key]; });
                }
            }

            if ((neighbor as any).predicates) {
                if(!builtins.has((neighbor as any).predicates.dgraph_type[0])) {
                    // Using 'any' because the PluginType is temporary, and not valid outside of the initial response
                    const pluginNode = {...(neighbor as any).predicates};
                    delete (neighbor as any).predicates
                    Object.keys(pluginNode).forEach(function(key) { (neighbor as any)[key] = pluginNode[key]; });
                }
            }
        })
    }
}