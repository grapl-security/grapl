import React from "react";

import NodeTable from '../nodeTable/NodeTable';
import { NodeDetailsProps, EngagementViewProps } from "types/LensAndNodeTableTypes";

import {ToggleLensTable} from "./utils/toggleLensTable";
import {ToggleNodeDetailTable} from './utils/toggleNodeDetailTable';

export const NodeDetails = ({node}: NodeDetailsProps) => {
    return ( 
        <> <NodeTable node={node} /> </> 
    )
}

export default function LensAndNodeTableContainer({setLens, curNode}: EngagementViewProps) {
    return (
        <>
            <ToggleLensTable setLens={setLens}/>
            <ToggleNodeDetailTable curNode={curNode}/>
        </>
    );
}