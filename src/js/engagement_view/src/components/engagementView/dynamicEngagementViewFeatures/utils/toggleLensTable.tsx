import React, {useEffect, useState} from "react";

import Button from "@material-ui/core/Button";
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import BubbleChartIcon from '@material-ui/icons/BubbleChart';
import Divider from "@material-ui/core/Divider";

import { pagedTable } from "./lensPagedTable";
import { getLenses}  from "../../../../services/graphQL/graphQlEndpointGetLensesReq";


import {
    ToggleLensTableProps, 
    ToggleLensTableState, 
} from "types/CustomTypes";

import { useStyles } from '../styles';

const defaultToggleLensTableState = (): ToggleLensTableState => {
    return {
        toggled: true,
        lenses: [],
        first: 100, // first is the page size
        offset: 0, // by default, start from page 0
    }
}


export function ToggleLensTable( {setLens}: ToggleLensTableProps ) {
    const [state, setState] = useState(defaultToggleLensTableState());
    const classes = useStyles();

    const [page, setPage] = useState(0);
    const [rowsPerPage, setRowsPerPage] = useState(10);
    const handleChangePage = (event: React.MouseEvent<HTMLButtonElement, MouseEvent> | null, page: number) => {
        setPage(page);
    }
    const handleChangeRowsPerPage = (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        console.log("Handle Row Event", event)
        setRowsPerPage(parseInt(event.target.value, 10));
        setPage(0);
    }

    useEffect( () => {
        const interval = setInterval(
            () => {
            console.log("Fetching lenses");

            getLenses(state.first, state.offset)
                .then((response) => {
                    if (response.lenses && response.lenses !== state.lenses) {
                        const lenses = state.lenses.concat(response.lenses);
                        setState({
                            ...state,
                            offset: state.offset + response.lenses.length || 0,
                            lenses,
                        })
                    }
                }
            )
        }, 5000);
        return () => clearInterval(interval);
    });

    return (
        <>
            <div className={classes.header}>
                <b className={classes.title}>
                    <BubbleChartIcon className = {classes.icon} />
                    LENSES 
                </b>
                <Button
                    className = {classes.button}
                    onClick={() => { 
                        setState({
                            ...state,
                            toggled: !state.toggled,
                        }) 
                    }}> 
                    <ExpandMoreIcon className={classes.expand}/> 
                </Button>
            </div>
        
            <div className="lensToggle">
                {   
                    state.toggled && 
                    pagedTable(state, page, rowsPerPage, handleChangePage, handleChangeRowsPerPage, setLens, classes)
                }
            </div>

            <Divider />
        </>
    )
}
