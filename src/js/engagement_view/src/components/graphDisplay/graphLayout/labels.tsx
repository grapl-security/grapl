import { NodeProperties } from "../../../types/CustomTypes";

const getNodeLabel = (nodeType: string, node: NodeProperties) => {
	const _node = node;
	return _node.display || nodeType;
};

const getLinkLabel = (labelType: string) => {
	if (labelType === "children") {
		return "executed";
	}
	return labelType;
};

export { getLinkLabel, getNodeLabel };
