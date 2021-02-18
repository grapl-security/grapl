import { NodeProperties } from "../../../types/CustomTypes";

export const mapNodeProps = (
	node: NodeProperties,
	f: (propName: string) => void
) => {
	for (const prop in node) {
		const nodeProp = node[prop];

		if (Object.prototype.hasOwnProperty.call(node, prop)) {
			if (Array.isArray(nodeProp)) {
				if (nodeProp.length > 0) {
					if (nodeProp[0].uid === undefined) {
						f(prop);
					}
				}
			} else {
				f(prop);
			}
		}
	}
};