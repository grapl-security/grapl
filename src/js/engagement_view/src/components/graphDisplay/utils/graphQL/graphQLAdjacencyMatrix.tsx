import { traverseNodes, traverseNeighbors, mapEdges } from "../graph/graph_traverse"
import { getNodeLabel } from '../graph/labels';
import {LensScopeResponse, BaseNode} from "../../../../types/CustomTypes"

interface IVizNode {
    uid: number,
    name: number,
    id: number, 
    nodeType: string,
    nodeLabel: string,
    x: number,
    y: number,
}

interface VizProcessNode extends IVizNode {
    process_id: number,
    process_name: string,
    created_timestamp: number, 
    terminate_time: number,
    image_name: string, 
    arguments: string,
}

export interface File extends IVizNode {
    file_name: string,
    file_path: string,
    file_extension: string,
    file_mime_type: string,
    file_size: number,
    file_version: string, 
    file_description: string,
    file_product: string,
    file_company: string, 
    file_directory: string,
    file_inode: number,
    file_hard_links: string, 
    signed: boolean,
    signed_status: string, 
    md5_hash: string,
    sha1_hash: string,
    sha256_hash: string,
}

export interface IpConnections extends IVizNode {
    src_ip_addr: string,
    src_port: string,
    dst_ip_addr: string,
    dst_port: string,
    created_timestamp: number,
    terminated_timestamp: number,
    last_seen_timestamp: number,
}
interface VizAssetNode extends IVizNode {
    hostname: string,
}

interface VizLensNode extends IVizNode {
    lens_name: string,
    lens_type: string, 
}

type VizDynamicNode = IVizNode;

type VizNode = VizProcessNode | VizAssetNode | VizLensNode | VizDynamicNode;

type VizLink = {
    source: number,
    label: string,
    target: number, 
}

type AdjacencyMatrix = {
    nodes: VizNode[],
    links: VizLink[]
}


const getNodeType = (node: BaseNode) => {
    const t = node.dgraph_type || node.node_type;

    if (t) {
        if (Array.isArray(t)) {
            return t[0]
        }
        return t
    }

    console.warn('Unable to find type for node ', node);
    return 'Unknown';
};

function randomInt(min: number, max: number) // min and max included
{
    let randomNum: number = Math.floor(Math.random() * (max - min + 1) + min);
    return randomNum;
}


export const graphQLAdjacencyMatrix = (inputGraph: (LensScopeResponse & BaseNode)): AdjacencyMatrix => {

    const nodes: VizNode[] = []; 
    const links: VizLink[] = [];

    const nodeMap: Map<number, VizNode> = new Map();

    traverseNeighbors(inputGraph, 
        (fromNode: BaseNode, edgeName: string, toNode: BaseNode) => {
            if(edgeName !== 'scope'){
                
                if(getNodeType(fromNode) === 'Unknown'){
                    return;
                }

                if(getNodeType(toNode) === 'Unknown'){
                    return;
                }

                if(getNodeType(fromNode) === 'Risk'){
                    return;
                }

                if(getNodeType(toNode) === 'Risk'){
                    return;
                }
                
                links.push({
                    source: fromNode.uid,
                    label: edgeName, 
                    target: toNode.uid
                })
        } 
    })

    traverseNodes(inputGraph, (node: BaseNode) => {
        const nodeType = getNodeType(node);

        if(nodeType === 'Unknown'){
            return;
        }

        if(nodeType === 'Risk'){
            return; 
        }

        const nodeLabel = getNodeLabel(nodeType, node);

        const strippedNode = {...node};

        strippedNode.risk = strippedNode.risk || 0;
        strippedNode.analyzer_names = strippedNode.analyzer_names || "";

        for(const risk of node.risks || []){
            strippedNode.risk += risk.risk_score || 0;
            if (strippedNode.analyzer_names && risk.analyzer_name) {
                // #TODO: Link to the analyzer details
                strippedNode.analyzer_names += ", "
            }
            strippedNode.analyzer_names += risk.analyzer_name || "";
        }

        mapEdges(node, (edge: string, _neighbor: BaseNode) => {
            // The stripped node is being converted to another type, so we can cast
            // to any here
            (strippedNode as any)[edge] = undefined;
        })

        const vizNode = {
            name: node.uid,
            x: 200 + randomInt(1, 5),
            y: 150 + randomInt(1, 5),
            ...strippedNode,
            id: node.uid,
            nodeType,
            nodeLabel,
        };

        nodeMap.set(node.uid, vizNode);
    })

    for (const vizNode of (nodeMap.values() as any)) {
        nodes.push(vizNode)
    }

    return {
        nodes, 
        links
    }
}
