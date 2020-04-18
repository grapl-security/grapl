// @ts-nocheck
import React, { useRef, useState, useEffect } from 'react';
import { ForceGraph2D, ForceGraph3D } from 'react-force-graph';
import * as d3 from "d3";

const engagement_edge = "http://localhost:8900/";

const BKDRHash = (str: any) => {
    const seed = 131;
    const seed2 = 137;
    let hash = 0 as any;
    // make hash more sensitive for short string like 'a', 'b', 'c'
    str += 'x';
    // Note: Number.MAX_SAFE_INTEGER equals 9007199254740991
    const MAX_SAFE_INTEGER = parseInt(9007199254740991 / seed2 as any) as any;
    for(let i = 0; i < str.length; i++) {
        if(hash > MAX_SAFE_INTEGER) {
            hash = parseInt(hash / seed2 as any);
        }
        hash = hash * seed + str.charCodeAt(i);
    }
    return hash;
};


/**
 * Convert HSL to RGB
 *
 * @see {@link http://zh.wikipedia.org/wiki/HSL和HSV色彩空间} for further information.
 * @param {Number} H Hue ∈ [0, 360)
 * @param {Number} S Saturation ∈ [0, 1]
 * @param {Number} L Lightness ∈ [0, 1]
 * @returns {Array} R, G, B ∈ [0, 255]
 */
const HSL2RGB = (H: any, S: any, L: any) => {
    H /= 360;

    const q = L < 0.5 ? L * (1 + S) : L + S - L * S;
    const p = 2 * L - q;

    return [H + 1/3, H, H - 1/3].map((color) => {
        if(color < 0) {
            color++;
        }
        if(color > 1) {
            color--;
        }
        if(color < 1/6) {
            color = p + (q - p) * 6 * color;
        } else if(color < 0.5) {
            color = q;
        } else if(color < 2/3) {
            color = p + (q - p) * 6 * (2/3 - color);
        } else {
            color = p;
        }
        return Math.round(color * 255);
    });
};

const isArray = (o: any) => {
    return Object.prototype.toString.call(o) === '[object Array]';
};

/**
 * Color Hash Class
 *
 * @class
 */
const ColorHash = function(options: any) {
    options = options || {};

    const LS = [options.lightness, options.saturation].map((param) => {
        param = param || [0.35, 0.5, 0.65]; // note that 3 is a prime
        return isArray(param) ? param.concat() : [param];
    });

    this.L = LS[0];
    this.S = LS[1];

    if (typeof options.hue === 'number') {
        options.hue = {min: options.hue, max: options.hue};
    }
    if (typeof options.hue === 'object' && !isArray(options.hue)) {
        options.hue = [options.hue];
    }
    if (typeof options.hue === 'undefined') {
        options.hue = [];
    }
    this.hueRanges = options.hue.map(function (range: any) {
        return {
            min: typeof range.min === 'undefined' ? 0 : range.min,
            max: typeof range.max === 'undefined' ? 360: range.max
        };
    });

    this.hash = options.hash || BKDRHash;
};

/**
 * Returns the hash in [h, s, l].
 * Note that H ∈ [0, 360); S ∈ [0, 1]; L ∈ [0, 1];
 *
 * @param {String} str string to hash
 * @returns {Array} [h, s, l]
 */
ColorHash.prototype.hsl = function(str: any) {
    let H, S, L;
    let hash = this.hash(str);

    if (this.hueRanges.length) {
        const range = this.hueRanges[hash % this.hueRanges.length];
        const hueResolution = 727; // note that 727 is a prime
        H = ((hash / this.hueRanges.length) % hueResolution) * (range.max - range.min) / hueResolution + range.min;
    } else {
        H = hash % 359; // note that 359 is a prime
    }
    hash = parseInt(hash / 360 as any);
    S = this.S[hash % this.S.length];
    hash = parseInt(hash / this.S.length as any);
    L = this.L[hash % this.L.length];

    return [H, S, L];
};

/**
 * Returns the hash in [r, g, b].
 * Note that R, G, B ∈ [0, 255]
 *
 * @param {String} str string to hash
 * @returns {Array} [r, g, b]
 */
ColorHash.prototype.rgb = function(str: any) {
    const hsl = this.hsl(str);
    return HSL2RGB.apply(this, hsl);
};



const retrieveGraph = async (lens: string) => {

    let uidHashes = {};

    // for (const node of graph.nodes) {
    //     if (node.lens !== undefined) {
    //         continue
    //     }
    //     if (node.uid !== undefined) {
    //         uidHashes[node.uid] = await hashNode(node);
    //     }
    // }

    console.log("Getting graph");

    const res = await fetch(`${engagement_edge}update`, {
        method: 'post',
        body: JSON.stringify({
            'lens': lens,
            'uid_hashes': uidHashes,
        }),
        headers: {
            'Content-Type': 'application/json',
        },
        credentials: 'include',
    });

    const json_res = await res.json();

    const updated_nodes = json_res['success']['updated_nodes'];
    const removed_nodes = json_res['success']['removed_nodes'];

    return [updated_nodes, removed_nodes]
};

const getNodeType = (node: any) => {
    if (node.process_id !== undefined) {
        return 'Process';
    }
    if (node.file_path !== undefined) {
        return 'File';
    }

    if (node.external_ip !== undefined) {
        return 'ExternalIp';
    }

    if (node.port !== undefined) {
        return 'Connect';
    }

    if (node.scope !== undefined || node.lens !== undefined) {
        return 'Lens';
    }

    // Dynamic nodes
    if (node.node_type) {
        return node.node_type[0]
    }

    console.warn('Unable to find type for node ', node);
    return 'Unknown';
};

function randomInt(min: number, max: number) // min and max included
{
    return Math.floor(Math.random() * (max - min + 1) + min);
}

const lensToAdjacencyMatrix = (matricies: any) => {
    const nodes = new Map();
    const links = new Map();

    const key_uid = new Map();

    for (const matrix of matricies) {
        const node_key = matrix.node['node_key'];
        const uid = matrix.node['uid'];
        if (matrix.node["analyzer_name"]) {
            continue
        }
        key_uid.set(node_key, uid);
        console.log(node_key);
        nodes.set(uid, matrix.node);
    }

    for (const matrix of matricies) {
        const node_key = matrix.node['node_key'];
        const uid = matrix.node['uid'];

        for (const edge of matrix.edges) {
            let edgeList = links.get(uid);
            const to_uid = key_uid.get(edge['to']);

            const edge_name = edge['edge_name'];
            if (edge_name === "risks") {
                const node = nodes.get(key_uid.get(edge['from']));
                for (const risk of matrix.node.risks) {
                    node.risk = risk.risk_score + (node.risk || 0)
                    if (node.analyzers) {
                        if (node.analyzers.indexOf(risk.analyzer_name) === -1) {
                            if (risk.analyzer_name) {
                                node.analyzers += ', ' + risk.analyzer_name
                            }
                        }
                    } else {
                        node.analyzers = risk.analyzer_name
                    }
                }

                continue
            }

            if (edgeList === undefined) {
                edgeList = new Map();
                edgeList.set(
                    uid +  + to_uid,
                    [uid, edge_name, to_uid]
                );

            } else {
                edgeList.set(
                    uid + edge_name + to_uid,
                    [uid, edge_name, to_uid]
                );
            }
            links.set(uid, edgeList)
        }
    }

    console.log(links)

    return {
        nodes, links
    }
};


const getNodeLabel = (nodeType: any, node: any) => {
    if (nodeType === 'Process') {
        return node.process_name || node.process_id;
    }

    if (nodeType === 'File') {
        return node.file_path;
    }

    if (nodeType === 'ExternalIp') {
        return node.external_ip;
    }

    if (nodeType === 'Connect') {
        return node.port;
    }

    if (nodeType === 'Lens') {
        return node.lens;
    }

    return nodeType || '';
};

const dgraphNodesToD3Format = (dgraphNodes: any) => {
    const graph = lensToAdjacencyMatrix(dgraphNodes) as any;

    // Calculate risks and attach to nodes
    for (const node of graph.nodes.values()) {
        const edges = graph.links.get(node.uid) || new Map();
        for (const edge of edges.values()) {
            if (edge[1] === 'risks') {
                const riskNode = graph.nodes.get(edge[2]);
                if (!riskNode.risk_score) {
                    continue
                }

                if (node.risk === undefined) {
                    node.risk = riskNode.risk_score;
                    node.analyzers = riskNode.analyzer_name;
                } else {
                    node.risk += riskNode.risk_score;
                    if (node.analyzers && node.analyzers.indexOf(riskNode.analyzer_name) === -1) {
                        node.analyzers += ', ' + riskNode.analyzer_name;
                    }
                }
            }
        }
    }

    // Flatten nodes
    const nodes = [];

    for (const node of graph.nodes.values()) {
        if (node.risk_score || node.analyzer_name) {
            continue
        }
        const nodeType = getNodeType(node);
        if (nodeType === 'Unknown') {
            continue
        }
        const nodeLabel = getNodeLabel(nodeType, node);
        nodes.push({
            name: node.uid,
            id: node.uid,
            ...node,
            nodeType,
            nodeLabel,
            x: 200 + randomInt(1, 50),
            y: 150 + randomInt(1, 50),
        });
    }

    // Flatten links
    const links = [];

    for (const linkMap_ of graph.links) {
        const nodeId = linkMap_[0];
        const node = graph.nodes.get(nodeId);

        if (node && node.lens) {
            // Don't link lens nodes, it's ugly
            continue
        }
        const linkMap = linkMap_[1];
        for (const links_ of linkMap.values()) {
            if (links_[1] === 'risks') {
                continue
            }
            if (links_[1] === '~scope') {
                continue
            }

            if (links_[1][0] === '~') {
                links.push({
                    source: links_[2],
                    label: links_[1].substr(1),
                    target: links_[0],
                });
            } else {
                links.push({
                    source: links_[0],
                    label: links_[1],
                    target: links_[2],
                });
            }

        }
    }

    return {
        nodes,
        links,
    };
};

const mapLabel = (label: any) => {
    if (label === 'children') {
        return 'executed'
    }
    return label
};


const percentToColor = (percentile: any) => {
    const hue = (100 - percentile) * 40 / 100;
    // if(percentile >= 75){
    //     // BRIGHT ASS RED
    // } else if (percentile >= 50){
    //     //YELLOW
    // } else if(percentile >=25) {
    //     //blue 
    // } else if(percentile > 0){
    //     // GREEN
    // } else {
    //     // more green
    // }

    return `hsl(${hue}, 100%, 50%)`;
};


const calcNodeRiskPercentile = (_nodeRisk: any, _allRisks: any) => {
    let nodeRisk = _nodeRisk;
    if (typeof _nodeRisk === 'object') {
        nodeRisk = _nodeRisk.risk;
    }

    const allRisks = _allRisks
        .map((n: any) => n || 0)
        .sort((a: any, b: any) => a - b);

    if (nodeRisk === undefined || nodeRisk === 0 || allRisks.length === 0) {
        return 0
    }

    let riskIndex = 0;
    for (const risk of allRisks) {
        if (nodeRisk >= risk) {
            riskIndex += 1;
        }
    }

    return Math.floor((riskIndex / allRisks.length) * 100)
};


const nodeSize = (node: any, Graph: any) => {
    const nodes = [...Graph.nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile >= 75) {
        return 6
    } else if (riskPercentile >= 25) {
        return 5
    } else {
        return 4
    }
};

const findNode = (id: any, nodes: any) => {
    for (const node of (nodes || [])) {
        if (node.id === id) {
            return node
        }
    }
    return null
};

// FILE TYPE COLORS
const calcNodeRgb = (node: any, colorHash: any) => {
    if (node.nodeType === 'Process') {
        return [31, 185, 128]
    } else if (node.nodeType === 'File') {
        return [177, 93, 255]
    } 
    
    // else if (node.nodeType === 'Lens'){
    //     return []
    // } else if(node.nodeType === 'IpPort'){
    //     return []
    // } else if(node.nodeType === 'IpConnection'){
    //     return []
    // } else if(node.nodeType === 'ProcessInboundConnection'){
    //     return []
    // } else if(node.nodeType === 'ProcessOutboundConnection'){
    //     return []
    // } 
    
    else{
        return colorHash.rgb(node.nodeType)
    }
}


const riskColor = (node: any, Graph: any, colorHash: any) => {
    const nodes = [...Graph.nodes].map(node => node.risk);

    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {
        const nodeColors = calcNodeRgb(node, colorHash);
        return `rgba(${nodeColors[0]}, ${nodeColors[1]}, ${nodeColors[2]}, 1)`;
    }

    return percentToColor(riskPercentile);
};

const calcLinkRisk = (link: any, Graph: any) => {
    let srcNode = findNode(link.source, Graph.nodes)
        || findNode(link.source.name, Graph.nodes);
    let dstNode = findNode(link.target, Graph.nodes)
        || findNode(link.target.name, Graph.nodes);

    if (srcNode === null) {
        srcNode = {risk: 0}
    }

    if (dstNode === null) {
        dstNode = {risk: 0}
    }

    const srcRisk = srcNode.risk || 0;
    const dstRisk = dstNode.risk || 0;

    return Math.round((srcRisk + dstRisk) / 2)
};

const calcLinkRiskPercentile = (link: any, Graph: any) => {
    const linkRisk = calcLinkRisk(link, Graph);
    const nodes = [...Graph.nodes].map(node => node.risk);

    return calcNodeRiskPercentile(linkRisk, nodes);
};

const calcLinkColor = (link: any, Graph: any) => {
    const risk = calcLinkRiskPercentile(link, Graph);
    // Default link color if no risk
    if (risk === 0) {return 'white'}
    return percentToColor(risk);
};


export const mapNodeProps = (node, f) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                if (node[prop].length > 0) {
                    if (node[prop][0].uid === undefined) {
                        f(prop)
                    }
                }
            } else {
                f(prop)
            }
        }
    }
};

const calcLinkParticleWidth = (link, Graph) => {
    const linkRiskPercentile = calcLinkRiskPercentile(link, Graph);
    if (linkRiskPercentile >= 75) {
        return 5
    } else if (linkRiskPercentile >= 50) {
        return 4
    }  else if (linkRiskPercentile >= 25) {
        return 3
    } else {
        return 2
    }
};

const calcLinkDirectionalArrowRelPos = (link, Graph) => {
    const node = findNode(link.target, Graph.nodes)
        || findNode(link.target.name, Graph.nodes);

    if (node === null || node.risk === 0) {
        return 1.0
    }
    const nodes = [...Graph.nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {return 1.0}

    if (riskPercentile >= 75) {
        return 0.95
    } else if (riskPercentile >= 50) {
        return 0.9
    } else if (riskPercentile >= 25) {
        return 0.85
    } else {
        return 1.0
    }
};



// merges y into x, returns true if update occurred
const mergeNodes = (x, y) => {
    let merged = false;
    mapNodeProps(y, (prop) => {
        if (!Object.prototype.hasOwnProperty.call(x, prop)) {
            merged = true;
            x[prop] = y[prop]
        }
    });

    return merged;
};


class GraphManager {
    constructor(graph) {
        this.graph = graph || {
            nodes: [], links: []
        };
    }

    updateNode = (newNode) => {
        console.log("newNode", newNode);
        if (newNode.uid === undefined) {return}
        for (let node of this.graph.nodes) {
            if (node.name === newNode.name) {
                return mergeNodes(node, newNode);
            }
        }
        console.log('adding new node');
        this.graph.nodes.push(newNode);
        return true;
    };

    updateLink(newLink) {
        console.log("newLink", newLink);
        for (const link of this.graph.links) {
            let src = link.source.name;
            if (src === undefined) {
                src = link.source;
            }

            let tgt = link.target.name;
            if (tgt === undefined) {
                tgt = link.target;
            }

            if (src === newLink.source) {
                if (tgt === newLink.target) {
                    // if (link.label === newLink.label) {

                    return false;
                    // }
                }
            }
        }

        this.graph.links.push(newLink);
        return true;
    }

    removeNode = (uid) => {
        for (let i = 0; i < this.graph.nodes.length; i++) {
            if (this.graph.nodes[i].uid === uid) {
                this.graph.nodes.splice(i, 1);
            }
        }
    };

    removeLink = (uid) => {
        for (let i = 0; i < this.graph.links.length; i++) {
            if (this.graph.links[i].source.uid === uid) {
                this.graph.links.splice(i, 1);
                continue
            }
            if (this.graph.links[i].target.uid === uid) {
                this.graph.links.splice(i, 1);
            }
        }
    };

    removeNodesAndLinks = (toRemove) => {
        for (const deadNode of toRemove) {
            this.removeNode(deadNode);
        }

        for (const deadLink of toRemove) {
            this.removeLink(deadLink);
        }

        // console.log("Removed nodes and links ", this.graph.nodes, this.graph.links);
    };

    updateGraph = (newGraph) => {
        if (newGraph.nodes.length === 0 && newGraph.links.length === 0) {
            return
        }

        if (newGraph === this.graph) {
            return
        }

        let updated = false;
        for (const newNode of newGraph.nodes) {
            if (this.updateNode(newNode)) {
                updated = true;
            }
        }

        for (const newLink of newGraph.links) {
            if (this.updateLink(newLink)) {
                updated = true;
            }
        }
        return updated;
    };

}

type LinkT = {

    source: string,
    label: string,
    target: string,
}

type GraphT = { 
    nodes: [any],
    links: [LinkT],
}

// #TODO: This algorithm is exponential, and doesn't have to be
const mergeGraphs = (curGraph: GraphT, update: graphT) => {
    // Merges two graphs into a new graph
    // returns 'null' if there are no updates to be made

    if (!update.nodes && !updates.links) {
        // empty update
        return null
    }

    const outputGraph = {nodes: [], links: []};

    let updated = false;

    const nodes = new Map();
    const links = new Map();

    for (const node of curGraph.nodes) {
        nodes.set(node.uid, node)
    }

    for (const newNode of update.nodes) {
        const node = nodes.get(newNode.uid);
        if (node) {
            if (mergeNodes(node, newNode)) {
                console.log("node, newNode", node, newNode);
                updated = true;
            }
        } else {
            nodes.set(newNode.uid, newNode);
            updated = true;
        }
    }

    for (const link of curGraph.links) {
        links.set(
            link.source + link.label + link.target,
            link,
        )
    }

    for (const newLink of update.links) {
        const link = links.get(newLink.source + newLink.label + newLink.target);

        if (link) {
            continue
        } else {
            links.set(newLink.source + newLink.label + newLink.target, newLink);
            updated = true;
        }
    }

    outputGraph.nodes = Array.from(nodes.values());
    outputGraph.links = Array.from(links.values());
    if (updated) {
        return outputGraph;
    } else {
        return null;
    }
}

const GraphDisplay = ({lensName, setCurNode}: any) => {
    const [state, setState] = React.useState({
        graphData: {nodes: [], links: []},
        last_update: Date.now(),
    });
    const forceRef = useRef(null);


    useEffect(() => {
        forceRef.current.d3Force("link", d3.forceLink());
        forceRef.current.d3Force('collide', d3.forceCollide(22));
        forceRef.current.d3Force("charge", d3.forceManyBody());
        forceRef.current.d3Force('box', () => {
            const N = 100;
            // console.log(Graph.width(), Graph.height())
            const SQUARE_HALF_SIDE = 20 * N * 0.5;
            state.graphData.nodes.forEach(node => {
                const x = node.x || 0, y = node.y || 0;
                // bounce on box walls
                if (Math.abs(x) > SQUARE_HALF_SIDE) { node.vx *= -1; }
                if (Math.abs(y) > SQUARE_HALF_SIDE) { node.vy *= -1; }
            });
        });

        if (lensName) {
            const now = Date.now();
            if (now - state.last_update <= 1_000) {
                return
            }

            retrieveGraph(lensName)
                .then(async ([updated_nodes, removed_nodes]) => {
                    const update = await dgraphNodesToD3Format(updated_nodes) as any;
                    // const graphManager = new GraphManager(state.graphData);
                    // #TODO: Merge our updates in rather than overwriting state,
                    // at which point we can remove this 'if' hack, which will break things
                    
                    const mergeUpdate = mergeGraphs(state.graphData, update);
                    console.log('update', update);
                    if (mergeUpdate !== null) {
                        setState({
                            ...state,
                            last_update: Date.now(),
                            graphData: mergeUpdate,
                        })    
                    }
                })
                .catch((e) => console.error("Failed to retrieveGraph ", e))
        }
    });
    console.log('GraphDisplay: ', lensName);

    // #TODO: We should be fetching this data from Grapl's API, see "retrieveGraph" in source
    const graphData = state.graphData;
    
    const colorHash = new ColorHash({});

    // #TODO: ADD ZOOM HANDLERS FOR MAX ZOOM IN/OUT


    return(
        <>
        
            <ForceGraph2D
                graphData={graphData}
                nodeLabel={(node: any) => node.nodeLabel}
                enableNodeDrag={true}
                linkDirectionalParticles = {1}
                linkDirectionalParticleWidth = {(link) => {
                    return calcLinkParticleWidth(link, graphData);
                }}
                linkDirectionalParticleColor = {(link) => {
                    return calcLinkColor(link, graphData)
                }}
                linkDirectionalParticleSpeed = {0.005}
                onNodeClick= {
                    (node: any, event: any) => {
                        console.log('clicked', node.nodeLabel);
                        setCurNode(node);
                    }
                }
                linkDirectionalArrowLength = {8}
                linkWidth = {4}
                linkDirectionalArrowRelPos = {(link => {
                    return calcLinkDirectionalArrowRelPos(link, graphData);
                })}
                linkCanvasObjectMode = {(() => 'after')}
                linkCanvasObject = {((link: any, ctx: any) => {
                    const MAX_FONT_SIZE = 8;
                    const LABEL_NODE_MARGIN = 8 * 1.5;
                    const start = link.source;
                    const end = link.target;
                    // ignore unbound links
                    link.color = calcLinkColor(link, graphData);
        
                    if (typeof start !== 'object' || typeof end !== 'object') return;
                    // calculate label positioning
                    const textPos = Object.assign(
                        ...['x', 'y'].map((c: any) => (
                            {
                                [c]: start[c] + (end[c] - start[c]) / 2 // calc middle point
                            }
                        )) as any
                    );

                    const relLink = { x: end.x - start.x, y: end.y - start.y };
        
                    const maxTextLength = Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) - LABEL_NODE_MARGIN * 8;
        
                    let textAngle = Math.atan2(relLink.y, relLink.x);
                    // maintain label vertical orientation for legibility
                    if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
                    if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

                    const label = mapLabel(link.label);
                    // estimate fontSize to fit in link length
                    ctx.font = '50px Arial';
                    const fontSize = Math.min(MAX_FONT_SIZE, maxTextLength / ctx.measureText(label).width);
                    ctx.font = `${fontSize + 5}px Arial`;
        
                    let textWidth = ctx.measureText(label).width;
        
                    textWidth += Math.round(textWidth * 0.25);
        
                    const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding
                    // draw text label (with background rect)
                    ctx.save();
                    ctx.translate(textPos.x, textPos.y);
                    ctx.rotate(textAngle);
                    ctx.fillStyle = 'rgb(115,222,255,1)';
                    ctx.fillRect(- bckgDimensions[0] / 2, - bckgDimensions[1] / 2, ...bckgDimensions);
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = 'white';
                    //content, left/right, top/bottom
                    ctx.fillText(label, .75, 3);
                    ctx.restore();
                })}
                nodeCanvasObject= {((node: any, ctx: any, globalScale: any) => {
                    // add ring just for highlighted nodes

                    const NODE_R = nodeSize(node, graphData);
                    ctx.save();
        
                    // Risk outline color
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, NODE_R * 1.3, 0, 2 * Math.PI, false);
                    ctx.fillStyle = riskColor(node, graphData, colorHash);
                    ctx.fill();
                    ctx.restore();
        
                    ctx.save();
        
                    // Node color
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, NODE_R * 1.2, 0, 2 * Math.PI, false);
        
                    const nodeRbg = calcNodeRgb(node, colorHash);
        
                    ctx.fillStyle = `rgba(${nodeRbg[0]}, ${nodeRbg[1]}, ${nodeRbg[2]}, 1)`;
                    ctx.fill();
                    ctx.restore();
        
                    const label = node.nodeLabel;
        
                    const fontSize = 15/globalScale;
        
                    ctx.font = `${fontSize}px Arial`;
                
        
                    const textWidth = ctx.measureText(label).width;
        
                    const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding
                    // node label color
                    ctx.fillStyle = 'rgba(48, 48, 48, 0.8)';
                    ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, ...bckgDimensions);
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
                    ctx.fillStyle = 'white';
                    ctx.fillText(label, node.x, node.y);
        
                })}
                ref={forceRef}
            />
        </>
    )
}

export default GraphDisplay;