import React, { useEffect, useState } from "react";

import Table from "@material-ui/core/Table";
import TableRow from "@material-ui/core/TableRow";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableContainer from "@material-ui/core/TableContainer";
import TableHead from "@material-ui/core/TableHead";
import TablePagination from "@material-ui/core/TablePagination";
import Button from "@material-ui/core/Button";
import DeleteOutlinedIcon from "@material-ui/icons/DeleteOutlined";
import { PluginTableState } from "../../../types/uploadPluginTypes";

import { getPluginList } from "../../../services/uploadPlugin/getPluginList";
import { deletePlugin } from "../../../services/uploadPlugin/deletePlugin";
import { useStyles } from "../uploadPluginStyles";

const defaultPluginTableState = (): PluginTableState => {
    return {
        rows: [],
        toggle: true,
    };
};

export const PluginTable = () => {
    const classes = useStyles();
    const [state, setState] = React.useState(defaultPluginTableState());
    const [page, setPage] = useState(0);
    const [rowsPerPage, setRowsPerPage] = useState(10);

    const handleChangePage = (
        event: React.MouseEvent<HTMLButtonElement, MouseEvent> | null,
        newPage: number
    ) => {
        setPage(newPage);
    };
    const handleChangeRowsPerPage = (
        event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
    ) => {
        setRowsPerPage(parseInt(event.target.value, 10));
        setPage(0);
    };

    const emptyRows =
        rowsPerPage -
        Math.min(rowsPerPage, state.rows.length - page * rowsPerPage);

    useEffect(() => {
        try {
            const interval = setInterval(async () => {
                await getPluginList().then((rows) => {
                    setState({
                        toggle: state.toggle,
                        rows,
                    });
                });
            }, 1000);
            return () => clearInterval(interval);
        } catch (e) {
            console.error("Unable to retrieve plugin list", e);
        }
    }, [state.toggle]);

    return (
        <>
            <TableContainer>
                <Table>
                    <TableHead>
                        <TableRow key={"plugin"}>
                            <TableCell align="left">PLUGINS</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody aria-label="PluginTable">
                        {state.rows
                            .slice(
                                page * rowsPerPage,
                                page * rowsPerPage + rowsPerPage
                            )
                            .map((pluginName: string) => {
                                return (
                                    <TableRow key={pluginName}>
                                        <TableCell align="right">
                                            {pluginName}
                                            <Button
                                                onClick={() => {
                                                    deletePlugin(
                                                        pluginName
                                                    ).then(() => {
                                                        setState({
                                                            ...state,
                                                            toggle:
                                                                state.toggle &&
                                                                false,
                                                        });
                                                        console.log(
                                                            "Plugin Deleted"
                                                        );
                                                        console.log("testing");
                                                    });
                                                }}
                                            >
                                                <DeleteOutlinedIcon
                                                    className={classes.btn}
                                                />
                                            </Button>
                                        </TableCell>
                                    </TableRow>
                                );
                            })}
                        {emptyRows > 0 && (
                            <TableRow style={{ height: 53 * emptyRows }}>
                                <TableCell colSpan={6} />
                            </TableRow>
                        )}
                    </TableBody>
                </Table>

                <TablePagination
                    aria-label="pagination"
                    rowsPerPageOptions={[5, 10, 25]}
                    component="div"
                    count={state.rows.length}
                    rowsPerPage={rowsPerPage}
                    page={page}
                    onPageChange={handleChangePage}
                    onChangeRowsPerPage={handleChangeRowsPerPage}
                />
            </TableContainer>
        </>
    );
};
