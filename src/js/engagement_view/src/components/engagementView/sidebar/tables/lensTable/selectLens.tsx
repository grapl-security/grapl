import React, { useState } from "react";

import TableCell from "@material-ui/core/TableCell";
import TableRow from "@material-ui/core/TableRow";

import { useStyles } from "./lensTableStyles";

import { SelectLensProps } from "types/LensAndNodeTableTypes";

export function SelectLens(props: SelectLensProps) {
    const classes = useStyles();

    return (
        <TableRow
            hover
            key={props.uid}
            onClick={() => {
                props.setLensTableState(props.uid);
                props.setLens(props.lens);
            }}
            selected={props.selectedId === props.uid}
            className={classes.tableRow}
        >
            <TableCell component="th" scope="row" align="left">
                {props.lens_type + " :  " + props.lens}
            </TableCell>

            <TableCell component="th" scope="row" align="right">
                {props.score}
            </TableCell>
        </TableRow>
    );
}
