import { NodeProperties } from "../../../types/CustomTypes";

export const getNodeLabel = (nodeType: string, node: NodeProperties) => {
    const _node = node;
    return _node.display || nodeType;
};
