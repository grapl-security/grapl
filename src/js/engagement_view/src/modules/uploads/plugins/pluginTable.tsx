import React, { useEffect, useState } from 'react';
import Table from '@material-ui/core/Table';
import TableRow from '@material-ui/core/TableRow';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TablePagination from '@material-ui/core/TablePagination';
import Button from "@material-ui/core/Button";
import DeleteOutlinedIcon from '@material-ui/icons/DeleteOutlined';
import { PluginTableState } from "../plugins/uploadPluginTypes"
import { getPluginList, deletePlugin} from "../plugins/apiRequests";
import { useStyles } from "../plugins/useStyles";

const defaultPluginTableState = (): PluginTableState => {
    return {
        rows: [],
        toggle: true,
    }
}

export const PluginTable = () => {
    const classes = useStyles();

    const [state, setState] = React.useState(defaultPluginTableState());
    const [page, setPage] = useState(0);
    const [rowsPerPage, setRowsPerPage] = useState(10);
    const handleChangePage = (event: any, newPage:any) => {
        setPage(newPage);
    }
    const handleChangeRowsPerPage = (event: any) => {
        setRowsPerPage(parseInt(event.target.value, 10));
        setPage(0);
    }

    useEffect(() => {
        getPluginList().then((rows) => {
            setState({
                ...state,
                rows
            })
        })
    }, [state.toggle])

    return(
        <>
            <TableContainer>
                <Table>
                    <TableHead>
                        <TableRow key = {"plugin"}>
                            <TableCell align = "left">PLUGINS</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {state.rows.map(
                            (pluginName: string) => {
                                return <TableRow key = { pluginName }> 
                                            <TableCell 
                                                align = "left"> 
                                                {pluginName} 
                                                <Button onClick={
                                                    () => { 
                                                        deletePlugin(pluginName)
                                                        .then( 
                                                            () => {
                                                                setState({
                                                                    ...state, 
                                                                    toggle: state.toggle && false
                                                                })
                                                                console.log("Plugin Deleted");
                                                            }
                                                        )
                                                    } 
                                                }
                                            >
                                            <DeleteOutlinedIcon className = {classes.btn}/></Button>
                                        </TableCell> 
                                    </TableRow>
                            }
                        )}
                    </TableBody>
                </Table>
                    <TablePagination
                        aria-label = "pagination"
                        rowsPerPageOptions={[5, 10, 25]}
                        component="div"
                        count={state.rows.length}
                        rowsPerPage={rowsPerPage}
                        page={page}
                        onChangePage={handleChangePage}
                        onChangeRowsPerPage={handleChangeRowsPerPage}
                    />
            </TableContainer>
        </>
    )
}