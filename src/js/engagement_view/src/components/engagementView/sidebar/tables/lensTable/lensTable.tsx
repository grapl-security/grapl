import React, { useState } from "react";

import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableContainer from "@material-ui/core/TableContainer";
import TablePagination from "@material-ui/core/TablePagination";
import TableHead from "@material-ui/core/TableHead";
import TableRow from "@material-ui/core/TableRow";
import TableCell from "@material-ui/core/TableCell";

import { ClassNameMap } from "@material-ui/styles/withStyles";
import { SelectLens } from "./selectLens";
import { Lens } from "types/CustomTypes";
import { PaginationState } from "types/CustomTypes";

export const lensTable = (
    paginationState: PaginationState,
    selectedIdState: number,
    setSelectedIdState: (selectedIdState: number) => void,
    page: number,
    rowsPerPage: number,
    handleChangePage: (
        event: React.MouseEvent<HTMLButtonElement, MouseEvent> | null,
        page: number
    ) => void,
    handleChangeRowsPerPage: (
        event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
    ) => void,
    setLens: (lens: string) => void,
    classes: ClassNameMap<string>
) => {
    return (
        <TableContainer className={classes.tableContainer}>
            <TablePagination
                className={classes.pagination}
                aria-label="pagination"
                rowsPerPageOptions={[5, 10, 25]}
                component="div"
                count={paginationState.lenses.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onPageChange={handleChangePage}
                onChangeRowsPerPage={handleChangeRowsPerPage}
            />
            <Table
                className={classes.table}
                aria-label="lens-table"
                key={"lensTable"}
            >
                <TableHead>
                    <TableRow>
                        <TableCell align="left">
                            <b> Lens Name </b>
                        </TableCell>
                        <TableCell align="right">
                            <b> Risk </b>
                        </TableCell>
                    </TableRow>
                </TableHead>

                <TableBody>
                    {paginationState.lenses
                        .slice(
                            page * rowsPerPage,
                            page * rowsPerPage + rowsPerPage
                        )
                        .map((lens: Lens) => {
                            return (
                                <SelectLens
                                    key={Number(lens.uid)}
                                    uid={lens.uid}
                                    lens={lens.lens_name}
                                    lens_type={lens.lens_type}
                                    score={lens.score}
                                    setLens={setLens}
                                    selectedId={selectedIdState}
                                    setLensTableState={setSelectedIdState}
                                />
                            );
                        })}
                </TableBody>
            </Table>
        </TableContainer>
    );
};
