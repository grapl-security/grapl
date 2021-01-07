import React from "react";

import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableContainer from "@material-ui/core/TableContainer";
import TablePagination from '@material-ui/core/TablePagination';

import { ClassNameMap } from '@material-ui/styles/withStyles';
import { SelectLens } from './selectLens';
import { Lens } from "types/CustomTypes";
import { PaginationState } from "types/CustomTypes";

export const pagedTable = (
    state: PaginationState, 
    page: number, 
    rowsPerPage: number, 
    handleChangePage: (event: React.MouseEvent<HTMLButtonElement, MouseEvent> | null, page: number) => void, 
    handleChangeRowsPerPage: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => void, 
    setLens: (lens: string) => void, 
    classes: ClassNameMap<string>
) => {
    return (
        <TableContainer>
            <TablePagination
                className = {classes.pagination}
                aria-label = "pagination"
                rowsPerPageOptions={[5, 10, 25]}
                component="div"
                count={state.lenses.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onChangePage={handleChangePage}
                onChangeRowsPerPage={handleChangeRowsPerPage}
            />
            {
                state.lenses 
                .slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                .map(
                    (lens: Lens) => {
                        return(
                            <Table className={classes.table} aria-label="lens table" key={Number(lens.uid)}>
                                <TableBody>
                                    <SelectLens 
                                        key={Number(lens.uid)}
                                        uid={lens.uid}
                                        lens={lens.lens_name}
                                        lens_type={lens.lens_type}
                                        score={lens.score}
                                        setLens={setLens}
                                    />
                                </TableBody>
                            </Table>
                        )
                    }
                )
            }
        </TableContainer>
    )
}