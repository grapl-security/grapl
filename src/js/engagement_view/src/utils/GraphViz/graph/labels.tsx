import { Node }  from '../CustomTypes'

const getNodeLabel = (nodeType: string, node: Node) => {
    console.log('nodetype', nodeType);

    const _node = node as any; // ignore-any

    switch(nodeType) {
        case "Process": return _node.process_name || _node.process_id || 'Process';
        case "Asset": return _node.hostname || 'Asset';
        case "File": return _node.file_path || 'File';
        case "IpAddress": return _node.external_ip || 'IpAddress';
        case "Lens":  return _node.lens_name || 'Lens';
        default: return nodeType || '';
    }
};

const mapLabel = (label: string) => {
    if (label === 'children') {
        return 'executed'
    }
    return label
};

export { mapLabel, getNodeLabel }