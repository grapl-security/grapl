import React from "react";

import NodeTable from '../nodeTable/NodeTable';
import { NodeDetailsProps, EngagementViewContentProps } from "../../../types/DynamicEngagementViewTypes";

import {ToggleLensTable} from "./utils/toggleLensTable";
import {ToggleNodeTable} from './utils/toggleNodeTable';

export const NodeDetails = ({node}: NodeDetailsProps) => {
    return ( 
        <> <NodeTable node={node} /> </> 
    )
}

export default function EngagementViewContent({setLens, curNode}: EngagementViewContentProps) {
    return (
        <>
            <ToggleLensTable setLens={setLens}/>
            <ToggleNodeTable curNode={curNode}/>
        </>
    );
}