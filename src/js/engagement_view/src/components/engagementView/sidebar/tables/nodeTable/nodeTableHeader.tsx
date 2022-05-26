import React from "react";

import TableCell from "@material-ui/core/TableCell";
import TableHead from "@material-ui/core/TableHead";
import TableRow from "@material-ui/core/TableRow";

import { VizNode } from "types/CustomTypes";

export function nodeTableHeader(node: VizNode, styles: any) {
    if (node) {
        return (
            <TableHead>
                <TableRow>
                    <TableCell align="left" className={styles.tableHeader}>
                        <b> Property </b>
                    </TableCell>
                    <TableCell align="left" className={styles.tableHeader}>
                        <b> Value </b>
                    </TableCell>
                </TableRow>
            </TableHead>
        );
    } else {
        return <></>;
    }
}
