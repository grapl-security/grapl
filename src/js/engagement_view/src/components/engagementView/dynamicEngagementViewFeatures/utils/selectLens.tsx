import React from "react";

import Button from "@material-ui/core/Button";
import TableCell from "@material-ui/core/TableCell";
import TableRow from "@material-ui/core/TableRow";

import { useStyles } from '../styles';

import { SelectLensProps } from "types/DynamicEngagementViewTypes"

export function SelectLens(props: SelectLensProps) {
    const classes = useStyles();
    return (
        <>
            <TableRow key={props.uid}>
                <TableCell component="th" scope="row">
                <Button className = {classes.lensName}
                    onClick={
                        () => { 
                            props.setLens(props.lens)    
                        }
                }>
                    {props.lens_type + " :\t\t" + props.lens + "\t\t" + props.score}
                </Button>
                </TableCell>
            </TableRow>
        </>
    )
}