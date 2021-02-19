import { VizGraph, Link, VizNode } from "../../../types/CustomTypes";
import { calcNodeRiskPercentile } from "./nodeCalcs";
import { riskOutline } from "./nodeColoring";

const findNode = (id: number, nodes: VizNode[]) => {
	for (const node of nodes || []) {
		if (node.id === id) {
			return node;
		}
	}
	return null;
};

export const calcLinkRisk = (link: Link, Graph: VizGraph) => {
		let sourceNode: any =
			findNode(link.source, Graph.nodes) || 
			findNode((link as any).source.name, Graph.nodes);
		let targetNode: any=
			findNode(link.target as any, Graph.nodes) ||
			findNode((link as any).target.name as any, Graph.nodes);

	const sourceRisk: number = (sourceNode.risk_score) || 0;
	const targetRisk: number = (targetNode.risk_score) || 0;

	if (!sourceNode || !targetNode) {
		console.error(
			"Missing srcNode/dstNode",
			sourceNode,
			link.source,
			targetNode,
			Graph.nodes
		);
		return 0;
	}
	return Math.round((sourceRisk + targetRisk) / 2);
};

export const calcLinkRiskPercentile = (link: Link, graph: VizGraph) => {
	const linkRisk = calcLinkRisk(link, graph);
	const nodes = [...graph.nodes].map((node) => node.risk);

	return calcNodeRiskPercentile(linkRisk, nodes);
};

export const calcLinkColor = (link: Link, graph: VizGraph): string => {	
    const risk = calcLinkRiskPercentile(link, graph);
	return riskOutline(risk) as string;   
};
	
export const calcLinkParticleWidth = (link: Link, graph: VizGraph): number => {
	const linkRiskPercentile = calcLinkRiskPercentile(link, graph);
	if (linkRiskPercentile >= 75) {
		return 5;
	} else if (linkRiskPercentile >= 50) {
		return 4;
	} else if (linkRiskPercentile >= 25) {
		return 3;
	} else {
		return 2;
	}
};

export const calcLinkDirectionalArrowRelPos = (link: Link, graph: VizGraph) => {
	const _link = link as any; 
    const node = 
        findNode(_link.target.uid, graph.nodes);

    if (node === null || node.risk === 0) {
        return 1.0
    }

	const nodes = [...graph.nodes].map(node => node.risk);
	
    const riskPercentile = calcNodeRiskPercentile(node.risk_score as any, nodes);

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