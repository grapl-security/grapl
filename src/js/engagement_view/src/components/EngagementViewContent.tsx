import React, {useEffect, useState} from "react";
import NodeTable from './NodeTable'
import Button from "@material-ui/core/Button";
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import BubbleChartIcon from '@material-ui/icons/BubbleChart';
import LensIcon from '@material-ui/icons/Lens';
import Divider from "@material-ui/core/Divider";
import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableContainer from "@material-ui/core/TableContainer";
import TableRow from "@material-ui/core/TableRow";
import { Lens } from "../modules/GraphViz/CustomTypes";
import { getGraphQlEdge } from "../modules/GraphViz/engagement_edge/getApiURLs";

import TablePagination from '@material-ui/core/TablePagination';
import { ClassNameMap } from '@material-ui/styles/withStyles';
import {SelectLensProps, ToggleLensTableProps, ToggleLensTableState, EngagementViewContentProps, NodeDetailsProps, ToggleNodeTableProps, PaginationState} from "../modules/GraphViz/CustomTypes"

import { useStyles } from './makeStyles/EngagementViewContentStyles';

function SelectLens(props: SelectLensProps) {
    const classes = useStyles();
    return (
        <>
            <TableRow key={props.uid}>
                <TableCell component="th" scope="row">
                <Button className = {classes.lensName}
                    onClick={
                        () => { 
                            props.setLens(props.lens)    
                        }
                }>
                    {/* #TODO: change color of lens name based on score */}
                    {props.lens_type + " :\t\t" + props.lens + "\t\t" + props.score}
                </Button>
                </TableCell>
            </TableRow>
        </>
    )
}

const defaultToggleLensTableState = (): ToggleLensTableState => {
    return {
        toggled: true,
        lenses: [],
        first: 100, // first is the page size
        offset: 0, // by default, start from page 0
    }
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
        }, 1000);
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

const graphql_edge = getGraphQlEdge();

const getLenses = async (first: number, offset: number) => {
    // console.log('fetching graph from', graphql_edge);

    const query = `
        {
            lenses(first: ${first}, offset: ${offset}) {
                uid,
                node_key,
                lens_name,
                score, 
                lens_type,
            }
        }
    `;
    // console.log(`connecting to: ${graphql_edge}graphql`);
    const res = await fetch(`${graphql_edge}graphQlEndpoint/graphql`,
        {
            method: 'post',
            body: JSON.stringify({ query: query }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        })
        .then(res => res.json())
        .then(res => {
            if (res.errors) {
                console.error("lenses failed", res.errors);
                res.data = {lenses: []};
            }
            return res
        })
        .then((res) => res.data);

        const jres = await res;
    return jres;
};

const NodeDetails = ({node}: NodeDetailsProps) => {
    return (
        <>
            <NodeTable node={node} />
        </>
    )
}

function ToggleNodeTable({curNode}: ToggleNodeTableProps) {
    const [toggled, toggle] = useState(true);
    const classes = useStyles();
    return (
        <>
        <div>
            <div className={classes.header}>
                <b className={classes.title}><LensIcon className={classes.icon}/> NODE</b>
                <Button
                    className = {classes.button}
                    onClick={
                        () => { toggle(toggled => !toggled) }
                    }> 	
                    <ExpandMoreIcon className={classes.expand}/> 
                </Button>
            </div>

            <div className="nodeToggle">
                {
                    toggled && curNode && 
                        <>
                            { <NodeDetails node={curNode}/> }
                        </>
                }
            </div>
        </div>
        </>
    )
}



export default function EngagementViewContent({setLens, curNode}: EngagementViewContentProps) {
    return (
        <>
            <ToggleLensTable setLens={setLens}/>
            <ToggleNodeTable curNode={curNode}/>
        </>
    );
}

