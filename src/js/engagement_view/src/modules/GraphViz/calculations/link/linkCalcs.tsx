import { calcNodeRiskPercentile } from '../node/nodeCalcs'; 
import { LinkType, VizNode, VizGraph } from '../../CustomTypes';


const findNode = (id: number, nodes: VizNode[]) => {
    for (const node of (nodes || [])) {
        if (node.id === id) {
            return node
        }
    }
    return null
};

export const calcLinkRisk = (link: LinkType, Graph: VizGraph) => {
    // console.log("LINK", link)
    let srcNode = 
        // findNode(link.source, Graph.nodes) || 
        findNode(link.source.name, Graph.nodes);
    let dstNode = 
    // findNode(link.target, Graph.nodes)||
        findNode(link.target.name, Graph.nodes);

    if (!srcNode || !dstNode) {
        console.error("Missing srcNode/dstNode", srcNode, link, dstNode);
        return 0;
    }

    const srcRisk = srcNode.risk || 0;
    const dstRisk = dstNode.risk || 0;

    return Math.round((srcRisk + dstRisk) / 2)
};

export const calcLinkDirectionalArrowRelPos = (link: LinkType, Graph: VizGraph) => {
    const node = 
    // findNode(link.target, Graph.nodes) || 
        findNode(link.target.name, Graph.nodes);

    if (node === null || node.risk === 0) {
        return 1.0
    }
    const nodes = [...Graph.nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {
        return 1.0
    }

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

export const calcLinkRiskPercentile = (link: LinkType, Graph: VizGraph) => {
    const linkRisk = calcLinkRisk(link, Graph);
    const nodes = [...Graph.nodes].map(node => node.risk);

    return calcNodeRiskPercentile(linkRisk, nodes);
};

export const calcLinkParticleWidth = (link: LinkType, Graph:VizGraph) => {
    const linkRiskPercentile = calcLinkRiskPercentile(link, Graph);
    if (linkRiskPercentile >= 75) {
        return 5
    } else if (linkRiskPercentile >= 50) {
        return 4
    } else if (linkRiskPercentile >= 25) {
        return 3
    } else {
        return 2
    }
};
