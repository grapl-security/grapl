import React, { useEffect } from 'react';
import Table from '@material-ui/core/Table';
import TableRow from '@material-ui/core/TableRow';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
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
            </TableContainer>
        </>
    )
}