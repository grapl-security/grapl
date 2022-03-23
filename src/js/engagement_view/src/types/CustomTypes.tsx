export type OtherNodeProperties = { [key: string]: string | number };
export type NodeEdges = { [key: string]: Node | Node[] };

export type BaseNodeProperties = {
    uid: number;
    node_key: string;
    dgraph_type: string[];
};

export type NodeProperties = BaseNodeProperties & OtherNodeProperties;

export type LensName = {
    lens_name: string;
};

export type Node = {
    risks?: Risk[];
} & NodeProperties &
    NodeEdges;

export type Risk = {
    analyzer_name: string;
    risk_score: number;
} & BaseNodeProperties;

export type Lens = {
    scope: Node[];
    lens_name: string;
    lens_type: string;
    score: number;
} & NodeProperties &
    NodeEdges;

export type Coordinates = {
    x?: number;
    y?: number;
    fx?: number;
    fy?: number;
    vx?: number;
    vy?: number;
};

export type VizNodeMeta = {
    id: number;
    nodeLabel: string;
    risk_score?: number;
    analyzer_names?: string[];
    neighbors?: VizNode[];
    links?: Link[];
} & Coordinates;

export type VizNode = VizNodeMeta & NodeProperties;

export type Link = {
    source: number;
    target: number;
    name: string;
};

export type VizGraph = {
    nodes: VizNode[];
    links: Link[];
    index: { [key: number]: VizNode };
};

export type LoginProps = {};

export type RouteState = {
    curPage: string;
    lastCheckLoginCheck: number;
};

export type SetRouteState = (routeState: RouteState) => void;

export type ToggleLensTableProps = {
    setLens: (lens: string) => void;
};

export type ToggleLensTableState = {
    toggled: boolean;
    lenses: Lens[];
    first: number;
    offset: number;
};

export type PaginationState = {
    first: number;
    lenses: Lens[];
    offset: number;
    toggled: boolean;
};
