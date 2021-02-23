import { NodeProperties, NodeEdges } from "types/CustomTypes";

const _mapGraph = <T extends NodeProperties & NodeEdges>(
	node: T,
	visited: Set<string>,
	f: (node: T, prop: string, neighbor: T) => void
) => {
	mapEdgeProps(node, (edgeName: string, neighbor: T) => {
		if (visited.has(node.uid + edgeName + neighbor.uid)) {
			return;
		}

		visited.add(node.uid + edgeName + neighbor.uid);

		f(node, edgeName, neighbor);
		_mapGraph(neighbor, visited, f);
	});
};

export const mapGraph = <T extends NodeProperties & NodeEdges>(
	node: T,
	f: (node: T, prop: string, neighbor: T) => void
) => {
	const visited: Set<string> = new Set();
	mapEdgeProps(node, (edgeName: string, neighbor: T) => {
		f(node, edgeName, neighbor);
		_mapGraph(neighbor, visited, f);
	});
};

// f Type = <T extends NodeProperties & NodeEdges>(prop:string, neighbor: BaseNodeProperties) => void)
// Given a node, call 'f' on any of its neighbors
export const mapEdges = <T extends NodeProperties & NodeEdges>(
	node: T,
	f: (prop: string, neighbor: T) => void
) => {
	for (const prop in node) {
		if (Object.prototype.hasOwnProperty.call(node, prop)) {
			const possibleNeighbor = node[prop] as any;
			if (Array.isArray(possibleNeighbor)) {
				for (const neighbor of possibleNeighbor) {
					if (neighbor.uid !== undefined) {
						f(prop, neighbor);
					}
				}
			} else {
				if (possibleNeighbor && possibleNeighbor.uid !== undefined) {
					f(prop, possibleNeighbor);
				}
			}
		}
	}
};

export const mapEdgeProps = <T extends NodeProperties & NodeEdges>(
	node: T,
	f: (prop: string, neighbor: T) => void
) => {
	for (const prop in node) {
		if (Object.prototype.hasOwnProperty.call(node, prop)) {
			const possibleNeighbor = (node as any)[prop];
			if (Array.isArray(possibleNeighbor)) {
				for (const neighbor of possibleNeighbor) {
					if (neighbor.uid !== undefined) {
						f(prop, neighbor);
					}
				}
			} else {
				if (possibleNeighbor && possibleNeighbor.uid !== undefined) {
					f(prop, possibleNeighbor);
				}
			}
		}
	}
};

export const traverseNodes = <T extends NodeProperties & NodeEdges>(
	node: T,
	callback: (node: T) => void
) => {
	callback(node);
	mapEdges(node, (_, neighbor) => {
		traverseNodes(neighbor, callback);
	});
};

export const traverseNeighbors = <T extends NodeProperties & NodeEdges>(
	node: T,
	callback: (node: T, prop: string, neighbor: T) => void
) => {
	mapEdges(node, (edgeName, neighbor) => {
		callback(node, edgeName, neighbor);
		traverseNeighbors(neighbor, callback);
	});
};
