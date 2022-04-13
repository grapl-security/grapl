import { mapNodeProps } from "./mapNodeProps";
import {
    Link,
    SummaryLink,
    VizGraph,
    VizNode,
} from "../../../types/CustomTypes";
import { summarizeLinks } from "./vizGraphFromLensScope";

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
    const [linksUpdated, links] = mergeSummarizedLinks(
        curGraph.links,
        updateGraph.links
    );
    updated = updated || linksUpdated;

    outputGraph.nodes = Array.from(nodes.values());
    outputGraph.links = links;

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

const mergeSummarizedLink = (
    oldLink: SummaryLink,
    newLink: SummaryLink
): [boolean, SummaryLink] => {
    const allInnerLinks = [...oldLink.innerLinks, ...newLink.innerLinks];
    const summarizedLink = summarizeLinks(allInnerLinks)[0];
    const updated =
        summarizedLink.innerLinks.length !== oldLink.innerLinks.length;

    return [updated, summarizedLink];
};

// We're normalizing the order because links are bidirectional.

const sortKeys = (link: Link) => {
    const key = [link.source, link.target];
    key.sort();

    const sKey = String(key[0]) + String(key[1]);

    return sKey;
};

const mergeSummarizedLinks = (
    oldLinks: SummaryLink[],
    newLinks: SummaryLink[]
): [boolean, SummaryLink[]] => {
    let updated = false;
    const mergedLinks: Map<string, SummaryLink> = new Map();

    for (const link of oldLinks) {
        const sKey = sortKeys(link);
        mergedLinks.set(sKey, link);
    }

    for (const link of newLinks) {
        const sKey = sortKeys(link);
        const oldLink = mergedLinks.get(sKey);

        if (oldLink) {
            const [didMerge, merged] = mergeSummarizedLink(oldLink, link);
            updated = updated || didMerge;
            mergedLinks.set(sKey, merged);
        } else {
            mergedLinks.set(sKey, link);
            updated = true;
        }
    }
    return [updated, Array.from(mergedLinks.values())];
};
