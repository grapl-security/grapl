import { VizNode } from "./CustomTypes";

export type SelectLensProps = {
    lens: string;
    score: number;
    uid: number;
    lens_type: string;
    setLens: (lens: string) => void;
};

export type NodeDetailsProps = {
    node: VizNode;
};

export type ToggleNodeTableProps = {
    curNode: VizNode | null;
};

export type EngagementViewProps = {
    setLens: (lens: string) => void;
    curNode: VizNode | null;
};
