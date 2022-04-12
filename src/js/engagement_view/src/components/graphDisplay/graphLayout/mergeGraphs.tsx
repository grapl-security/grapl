import { mapNodeProps } from "./mapNodeProps";
import { Link, VizGraph, VizNode } from "../../../types/CustomTypes";

// if graph has updated, merge y into x
export const mergeNodes = (node: VizNode, newNode: VizNode) => {
    let merged = false;
    let nodeWithoutVizFormatting = {} as VizNode;
    for (const prop in node) {
        if (
            prop === "fx" ||
            prop === "fy" ||
            prop === "links" ||
            prop === "neighbors" ||
            prop === "vx" ||
            prop === "vy" ||
            prop === "x" ||
            prop === "y" ||
            prop === "vx" ||
            prop === "vy"
        ) {
            continue;
        }

        nodeWithoutVizFormatting[prop] = node[prop];
    }

    const _node = nodeWithoutVizFormatting;

    mapNodeProps(newNode, (prop: string) => {
        if (!Object.prototype.hasOwnProperty.call(_node, prop)) {
            if ((_node as any)[prop] !== (newNode as any)[prop]) {
                (_node as any)[prop] = (newNode as any)[prop];
                merged = true;
            }
        }
        return;
    });
    return merged;
};

export const mergeGraphs = (
    curGraph: VizGraph,
    updateGraph: VizGraph
): VizGraph | null => {
    // Merges two graphs into a new graph, returns 'null' when there are no new updates

    if (!updateGraph.nodes && !updateGraph.links) {
        return null;
    }

    let updated = false;

    const outputGraph: VizGraph = { nodes: [], links: [], index: {} };
    const nodes = new Map();
    const links = new Map();

    for (const node of curGraph.nodes) {
        nodes.set(node.uid, node);
    }

    for (const newNode of updateGraph.nodes) {
        const node = nodes.get(newNode.uid);
        if (node) {
            if (mergeNodes(node, newNode)) {
                updated = true;
            }
        } else {
            nodes.set(newNode.uid, newNode);
            updated = true;
        }
    }

    for (const link of curGraph.links) {
        if (link) {
            const _link = link as any;
            const setLink = _link.source.uid + link.name + _link.target.uid;
            links.set(setLink, link);
        }
    }

    for (const newLink of updateGraph.links) {
        const getLink = newLink.source + newLink.name + newLink.target;
        const link = links.get(getLink);

        if (!link) {
            links.set(newLink.source + newLink.name + newLink.target, newLink);
            updated = true;
        }
    }

    outputGraph.nodes = Array.from(nodes.values());
    outputGraph.links = Array.from(links.values());

    for (const node of outputGraph.nodes) {
        outputGraph.index[node.uid] = node;
    }

    outputGraph.links.forEach((link) => {
        // the graph should not be updated if the link is already in the index array

        const sourceNode = outputGraph.index[link.source] as any;
        const targetNode = outputGraph.index[link.target] as any;

        if (sourceNode === undefined || !targetNode === undefined) {
            return;
        }

        !sourceNode.neighbors && (sourceNode.neighbors = new Set());
        !targetNode.neighbors && (targetNode.neighbors = new Set());

        sourceNode.neighbors.add(targetNode);
        targetNode.neighbors.add(sourceNode);

        !sourceNode.links && (sourceNode.links = new Set());
        !targetNode.links && (targetNode.links = new Set());

        sourceNode.links.add(link);
        targetNode.links.add(link);
    });

    if (updated) {
        return outputGraph;
    } else {
        return null;
    }
};

type SummaryLink = {
    source: number;
    target: number;
    name: string;
    innerLinks: Link[];
};

const mergeLinks = (links: Link[]): SummaryLink[] => {
    const mergedLinks: Map<number[], Link[]> = new Map();
    const newLinks: SummaryLink[] = [];

    // TODO: create new type called inner link that has a source, target, and name
    //  to preserve directionality. instead of a string[], store an [{}] containing name and original source/destination for Link
    for (const link of links) {
        const key = [link.source, link.target];
        key.sort();

        const names = mergedLinks.get(key) || [];

        names.push(link);

        mergedLinks.set(key, names);
    }

    // TODO: add names to Link type and pass in []
    for (const [key, innerLinks] of mergedLinks.entries()) {
        newLinks.push({
            source: key[0],
            target: key[1],
            name: innerLinks[0].name,
            innerLinks: innerLinks,
        });
    }

    return newLinks;
};
