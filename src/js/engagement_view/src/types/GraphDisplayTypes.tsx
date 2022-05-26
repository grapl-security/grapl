import { VizGraph, VizNode } from "./CustomTypes";

export type GraphDisplayProps = {
    lensName: string | null;
    setCurNode: (node: VizNode) => void;
};

export type GraphDisplayState = {
    graphData: VizGraph;
    curLensName: string | null;
};

export type GraphState = {
    curLensName: string;
    graphData: VizGraph;
};
