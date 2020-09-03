import {mapNodeProps} from '../../../../src/components/GraphViz';
import {Node, MergeGraphType} from '../CustomTypes'; 

// merges y into x, returns true if update occurred
const mergeNodes = (x: Node, y: Node) => {
    let merged = false;
    mapNodeProps(y, (prop: string) => {
        if (!Object.prototype.hasOwnProperty.call(x, prop)) {
            if ((x as any)[prop] !== (y as any)[prop]) {
                (x as any)[prop] = (y as any)[prop];
                merged = true;
            }
        }
    });

    return merged;
};

export const mergeGraphs = (curGraph: MergeGraphType, update: MergeGraphType): MergeGraphType | null => {
    // Merges two graphs into a new graph
    // returns 'null' if there are no updates to be made

    if (!update.nodes && !update.links) {
        // empty update
        return null
    }

    const outputGraph: MergeGraphType = {nodes: [], links: []};

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
                updated = true;
            }
        } else {
            nodes.set(newNode.uid, newNode);
            console.log('new node added ', newNode);
            updated = true;
        }
    }
// #TODO: console.log on link.source, check to see if it's an object or not. It should never be an object
// this should only be a string / an int. At some point, it was getting sent as an object
    for (const link of curGraph.links) {
        if (link) {
            const source = link.source.uid || link.source;
            const target = link.target.uid || link.target;
            links.set(
                source + link.label + target,
                link,
            )
        }
    }

    for (const newLink of update.links) {
        const newLinkSource =  newLink.source || newLink.source;
        const newLinkTarget =  newLink.target || newLink.target;
        const link = links.get(newLinkSource + newLink.label + newLinkTarget);
        if (!link) {
            console.log('newlink', newLink)
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