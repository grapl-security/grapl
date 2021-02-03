import { calcNodeRiskPercentile } from '../calculations/node/nodeCalcs';
import { calcLinkRiskPercentile } from '../calculations/link/linkCalcs';
import { LinkType, VizGraph, VizNode } from '../CustomTypes'
import { ColorHash } from '../../../components/GraphViz';

export const BKDRHash = (str: string) => {
    const seed = 131;
    const seed2 = 137;
    let hash = 0 as number;
    // make hash more sensitive for short string like 'a', 'b', 'c'
    str += 'x';
    // Note: Number.MAX_SAFE_INTEGER equals 9007199254740991
    const MAX_SAFE_INTEGER = parseInt(9007199254740991 / seed2 as any) as any;
    for (let i = 0; i < str.length; i++) {
        if (hash > MAX_SAFE_INTEGER) {
            hash = parseInt(hash / seed2 as any);
        }
        hash = hash * seed + str.charCodeAt(i);
    }
    return hash;
};

//# TODO: Add custom coloring for each node
export const calcNodeRgb = (node: VizNode, colorHash: ColorHash) => {
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


export const percentToColor = (percentile: number) => {
    const hue = (100 - percentile) * 40 / 100;

    return `hsl(${hue}, 100%, 50%)`;
};

export const calcLinkColor = (link: LinkType, Graph: VizGraph) => {
    const risk = calcLinkRiskPercentile(link, Graph);
    // Default link color if no risk
    if (risk === 0) {
        return 'white'
    }
    return percentToColor(risk);
};


export const riskColor = (node: VizNode, Graph: VizGraph, colorHash: ColorHash) => {
    const nodes = [...Graph.nodes].map(node => node.risk);

    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {
        const nodeColors = calcNodeRgb(node, colorHash);
        return `rgba(${nodeColors[0]}, ${nodeColors[1]}, ${nodeColors[2]}, 1)`;
    }

    return percentToColor(riskPercentile);
};


