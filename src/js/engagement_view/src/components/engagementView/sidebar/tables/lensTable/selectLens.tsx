import React, { useState } from "react";

import Button from "@material-ui/core/Button";
import TableCell from "@material-ui/core/TableCell";
import TableRow from "@material-ui/core/TableRow";

import { useStyles } from "./lensTableStyles";

import { SelectLensProps } from "types/LensAndNodeTableTypes";

export function SelectLens(props: SelectLensProps) {
    const classes = useStyles();

    const [btnClass, setBtnClass] = useState(false);

    return (
        <TableRow className={classes.tableRow} key={props.uid}>
            <TableCell component="th" scope="row" align="left">
                <Button
                    // className={classes.lensName}
                    className={btnClass ? "btnClass clicked" : "btnClass"}
                    onClick={() => {
                        props.setLens(props.lens);
                        btnClass ? setBtnClass(false) : setBtnClass(true);
                    }}
                >
                    {props.lens_type + " :\t\t" + props.lens}
                </Button>
            </TableCell>

            <TableCell component="th" scope="row" align="right">
                {props.score}
            </TableCell>
        </TableRow>
    );
}
