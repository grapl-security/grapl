import { NodeProperties } from "../../../types/CustomTypes";

const getNodeLabel = (nodeType: string, node: NodeProperties) => {
    const _node = node;
    return _node.display || nodeType;
};

const getLinkLabel = (labelType: string) => {
    return labelType;
};

export { getNodeLabel, getLinkLabel };
