


// Given a node, call 'f' on any of its neighbors
const mapEdges = (node: any, f: any) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                for (const neighbor of node[prop]) {
                    if (neighbor.uid !== undefined) {
                        f(prop, neighbor)
                    }
                }
            }
        }
    }
};


const traverseNodes = (node: any, callback: any) => {
    callback(node);
    mapEdges(node, (_: any, neighbor: any) => {
        traverseNodes(neighbor, callback);
    })
}

const traverseNeighbors = (node: any, callback: any) => {
    mapEdges(node, (edgeName: any, neighbor: any) => {
        callback(node, edgeName, neighbor);

        traverseNeighbors(neighbor, callback);
    })
}

export {traverseNodes, traverseNeighbors, mapEdges};