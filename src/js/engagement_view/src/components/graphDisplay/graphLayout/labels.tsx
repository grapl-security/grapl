import { NodeProperties } from "../../../types/CustomTypes";

const getNodeLabel = (nodeType: string, node: NodeProperties) => {
	const _node = node;

	switch (nodeType) {
		case "Process":
			return _node["process_name"] || _node["process_id"] || "Process";
		case "Asset":
			return _node["hostname"] || "Asset";
		case "File":
			return _node["file_path"] || "File";
		case "IpAddress":
			return _node["external_ip"] || "IpAddress";
		case "Lens":
			return _node["lens_name"] || "Lens";
		default:
			return nodeType || "";
	}
};

const getLinkLabel = (labelType: string) => {
	if (labelType === "children") {
		return "executed";
	}
	return labelType;
};

export { getLinkLabel, getNodeLabel };
