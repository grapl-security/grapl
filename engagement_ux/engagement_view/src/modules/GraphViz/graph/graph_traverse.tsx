import { Node } from "../CustomTypes";

const _mapGraph = (node: Node, visited: Set<string>, f:(node:Node, prop:string, neighbor: Node) => void) => {
    mapEdgeProps(node, (edgeName: string, neighbor:Node) => {
        if (visited.has(node.uid + edgeName + neighbor.uid)) {
            return
        }

        visited.add(node.uid + edgeName + neighbor.uid);

        f(node, edgeName, neighbor);
        _mapGraph(neighbor, visited, f)
    })
};

export const mapGraph = (node:Node, f:(node:Node, prop:string, neighbor: Node) => void) => {
    const visited: Set<string> = new Set();
    mapEdgeProps(node, (edgeName:string , neighbor:Node) => {

        f(node, edgeName, neighbor);
        _mapGraph(neighbor, visited, f)
    })
};

// type fType = (prop:string, neighbor: Node) => void)
// Given a node, call 'f' on any of its neighbors
export const mapEdges = (node: Node, f: (prop:string, neighbor: Node) => void) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray((node as any)[prop])) {
                for (const neighbor of (node as any)[prop]) {
                    if (neighbor.uid !== undefined) {
                        f(prop, neighbor)
                    }
                }
            }
        }
    }
};

export const mapEdgeProps = (node: Node, f: (prop:string, neighbor: Node) => void) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray((node as any)[prop])) {
                for (const neighbor of (node as any)[prop]) {
                    if (neighbor.uid !== undefined) {
                        f(prop, neighbor)
                    }
                }
            }
        }
    }
};

export const traverseNodes = (node: Node, callback: (node: Node) => void) => {
    callback(node);
    mapEdges(node, (_, neighbor) => {
        traverseNodes(neighbor, callback);
    })
}

export const traverseNeighbors = (node: Node, callback: (node:Node, prop:string, neighbor: Node) => void) => {
    mapEdges(node, (edgeName: string, neighbor: Node) => {
        callback(node, edgeName, neighbor);

        traverseNeighbors(neighbor, callback);
    })
}

