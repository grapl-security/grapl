export const baseNodeData = {
    uid: 50133,
    dgraph_type: ["Risk"],
    node_key: "Suspicious svchost",
    analyzer_name: "Suspicious svchost",
    risk_score: 75,
};

export const linkData = { source: 40, name: "asset_processes", target: 171 };

export const processNode = {
    uid: 207,
    node_key: "739c609a-05ac-4bed-949c-387efb06114a",
    dgraph_type: ["Process"],
    display: null,
    process_name: "svchost.exe",
    process_id: 6132,
};

export const rawNode: any  = {
    analyzerNames: "Rare Parent of cmd.exe, Suspicious svchost",
    asset_ip: null,
    asset_processes: undefined,
    dgraph_type: ["Asset"],
    display: "DESKTOP-FVSHABR",
    files_on_asset: null,
    fx: 7.039093729879062,
    fy: -0.4117743026991536,
    hostname: "DESKTOP-FVSHABR",
    id: 10057,
    index: 0,
    last_index_time: null,
    name: 10057,
    nodeLabel: "DESKTOP-FVSHABR",
    nodeType: "Asset",
    node_key: "e52e45e0-7ebf-4539-a66c-8b580441d101",
    risk_score: 85,
    risks: undefined,
    uid: 10057,
    vx: 0,
    vy: 0,
    x: 7.039093729879062,
    y: -0.4117743026991536,
    __indexColor: "#ec0001",
};

export const displayNodeData = {
    analyzerNames: "Rare Parent of cmd.exe, Suspicious svchost",
    hostname: "DESKTOP-FVSHABR",
    node_key: "e52e45e0-7ebf-4539-a66c-8b580441d101",
    risk_score: 85,
};

export const hidden = new Set([
    "id",
    "dgraph.type",
    "dgraph_type",
    "__indexColor",
    "risks",
    "uid",
    "scope",
    "name",
    "nodeType",
    "nodeLabel",
    "x",
    "y",
    "index",
    "vy",
    "vx",
    "fx",
    "fy",
    "links",
    "neighbors",
    "display",
]);
