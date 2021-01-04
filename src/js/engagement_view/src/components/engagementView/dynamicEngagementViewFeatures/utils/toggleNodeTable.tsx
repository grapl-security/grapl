import React, {useState} from "react";

import Button from "@material-ui/core/Button";
import LensIcon from '@material-ui/icons/Lens';
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';

import {
    ToggleNodeTableProps,
} from "../../../../types/DynamicEngagementViewTypes";


import {NodeDetails} from '../DynamicEngagementViewFeatures'

import { useStyles } from '../styles';


export function ToggleNodeTable({curNode}: ToggleNodeTableProps) {
    const [toggled, toggle] = useState(true);
    const classes = useStyles();
    return (
        <>
        <div>
            <div className={classes.header}>
                <b className={classes.title}><LensIcon className={classes.icon}/> NODE</b>
                <Button
                    className = {classes.button}
                    onClick={
                        () => { toggle(toggled => !toggled) }
                    }> 	
                    <ExpandMoreIcon className={classes.expand}/> 
                </Button>
            </div>

            <div className="nodeToggle">
                {
                    toggled && curNode && 
                        <>
                            { <NodeDetails node={curNode}/> }
                        </>
                }
            </div>
        </div>
        </>
    )
}