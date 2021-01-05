import React from "react";

import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';

import { Node } from "../../../types/CustomTypes";

export function tableHeader(node: Node, styles: any) {
    if(node) {
        return (
        <TableHead >
            <TableRow>
                <TableCell 
                    align="left" 
                    className={styles.tableHeader}>
                    <b> PROPERTY </b>
                </TableCell>
                <TableCell 
                    align="left"
                    className={styles.tableHeader}
                >
                    <b> VALUE </b>
                </TableCell>
            </TableRow>
        </TableHead>
    )
    } else {
        return <></>
    }
}
