import React, {useEffect, useState} from "react";

import Button from "@material-ui/core/Button";

import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import BubbleChartIcon from '@material-ui/icons/BubbleChart';

import Divider from "@material-ui/core/Divider";
import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableContainer from "@material-ui/core/TableContainer";
import TablePagination from '@material-ui/core/TablePagination';
import { ClassNameMap } from '@material-ui/styles/withStyles';
import NodeTable from '../nodeTable/NodeTable';

import { NodeDetailsProps } from "./types";

import { Lens, Node } from "components/graphViz/utils/GraphVizCustomTypes";

import {getLenses} from "../../../apiRequests/graphQlEndpointGetLensesReq";

import {ToggleNodeTable} from './utils/toggleNodeTable';
import {
    ToggleLensTableProps, 
    ToggleLensTableState, 
    PaginationState
} from "components/graphViz/utils/GraphVizCustomTypes";

import {SelectLens} from './utils/selectLens';

import { useStyles } from './styles';



type EngagementViewContentProps = {
    setLens: (lens: string) => void, 
    curNode: Node | null
}

const defaultToggleLensTableState = (): ToggleLensTableState => {
    return {
        toggled: true,
        lenses: [],
        first: 100, // first is the page size
        offset: 0, // by default, start from page 0
    }
}
export const NodeDetails = ({node}: NodeDetailsProps) => {
    return (
        <>
            <NodeTable node={node} />
        </>
    )
}

const pagedTable = (
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

function ToggleLensTable( {setLens}: ToggleLensTableProps ) {
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

// const graphql_edge = getGraphQlEdge();

// const getLenses = async (first: number, offset: number) => {
//     // console.log('fetching graph from', graphql_edge);

//     const query = `
//         {
//             lenses(first: ${first}, offset: ${offset}) {
//                 uid,
//                 node_key,
//                 lens_name,
//                 score, 
//                 lens_type,
//             }
//         }
//     `;

//     console.log("calling graphql_edge: " + graphql_edge + "with query: " + query);
    
//     const res = await fetch(`${graphql_edge}graphQlEndpoint/graphql`,
//         {
//             method: 'post',
//             body: JSON.stringify({ query: query }),
//             headers: {
//                 'Content-Type': 'application/json',
//             },
//             credentials: 'include',
//         })
//         .then(res => res.json())
//         .then(res => {
//             if (res.errors) {
//                 console.error("lenses failed", res.errors);
//                 res.data = {lenses: []};
//             }
//             return res
//         })
//         .then((res) => res.data);

//         const jres = await res;

//         console.log("queried graphql_edge in engagement view content", jres);
//     return jres;
// };

export default function EngagementViewContent({setLens, curNode}: EngagementViewContentProps) {
    return (
        <>
            <ToggleLensTable setLens={setLens}/>
            <ToggleNodeTable curNode={curNode}/>
        </>
    );
}

