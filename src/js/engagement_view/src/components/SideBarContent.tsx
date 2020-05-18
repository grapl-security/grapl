import React, {useEffect, useState} from "react";
import NodeTable from './NodeTable'
import Button from "@material-ui/core/Button";
import {makeStyles} from "@material-ui/core/styles";
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import BubbleChartIcon from '@material-ui/icons/BubbleChart';
import LensIcon from '@material-ui/icons/Lens';
import Divider from "@material-ui/core/Divider";
import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableContainer from "@material-ui/core/TableContainer";
import TableRow from "@material-ui/core/TableRow";
import {mapEdgeProps} from '../modules/GraphViz/graph/graph_traverse'; 
import {Node, Lens} from "../modules/GraphViz/CustomTypes";
import {getGraphQlEdge} from "../modules/GraphViz/engagement_edge/getEngagementEdge";

const useStyles = makeStyles({
    root:{
        fontSize: "1rem",
    },
    button: {
        width: ".005%",
        color: "white",
        backgroundColor:"#424242",
    },
    title: {
        fontSize: "25px",
        color: "#ffffff",
    },
    icon:{
        color: "#42C6FF",
        margin: "15px 0 0 10px",
    }, 
    expand:{
        color: "#42C6FF",
        margin: "0px"
    },
    header:{
        display: "flex"
    }, 
    table: {
        minWidth: 450
    },
});

type SelectLensProps = {
    lens: string,
    score: number,
    uid: number,
    setLens: (lens: string) => void,
}

function SelectLens(props: SelectLensProps) {
    // lensRows.push(createData(props.setLens(props.lens) ))
    return (
        <>
                <TableRow key={props.lens}>
                        <TableCell component="th" scope="row">
                        <Button 
                            onClick={
                                () => { 
                                    props.setLens(props.lens)    
                                }
                        }>
                            {props.lens + "\t\t" + props.score}
                        </Button>
                        </TableCell>
                    </TableRow>
        </>
    )
}


type ToggleLensTableProps = {
    setLens: (lens: string) => void,
}

type ToggleLensTableState = {
    toggled: boolean,
    lenses: Lens[],
}

const defaultToggleLensTableState = (): ToggleLensTableState => {
    return {
        toggled: true,
        lenses: [],
    }
}

function ToggleLensTable({setLens}: ToggleLensTableProps) {
    const [state, setState] = useState(defaultToggleLensTableState());

    const classes = useStyles();

    useEffect(() => {
        const interval = setInterval(() => {
            console.log("Fetching lenses");
            getLenses()
                .then((response) => {
                    if (response.lenses && response.lenses !== state.lenses) {
                        setState({
                            ...state,
                            lenses: response.lenses || [],
                        })
                    }
                })
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
                {state.toggled && state.lenses &&
                    state.lenses.map(
                        (lens: Lens) => {
                            // lensRows.push(lens);
                            return(
                                <TableContainer>
                                    <Table className={classes.table} aria-label="lens table">
                                        <TableBody>
                                            <SelectLens 
                                                key={Number(lens.uid)}
                                                uid={lens.uid}
                                                lens={lens.lens_name}
                                                score={lens.score}
                                                setLens={setLens}
                                            />
                                        </TableBody>
                                    </Table>
                                </TableContainer>
                            )
                        }
                    )
                }
            </div>
            
            <Divider />
        </>
    )
}

// const engagement_edge = getEngagementEdge();
const graphql_edge = getGraphQlEdge();


const getLenses = async () => {
    console.log('fetching graph from', graphql_edge);

    const query = `
    {
        lenses {
            uid,
            node_key,
            lens_name,
            score
        }
    }
    `;
    console.log('connecting to: ' + `${graphql_edge}graphql`)
    const res = await fetch(`${graphql_edge}graphql`,
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

type NodeDetailsProps = {
    node: Node
}

const NodeDetails = ({node}: NodeDetailsProps) => {
    // #TODO: Remove hidden fields from our node before displaying
    // Display remaining fields of node in our component="div"

    return (
        <>
            <NodeTable node={node} />
        </>
    )
}

type ToggleNodeTableProps = {
    curNode: Node | null
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


type SideBarContentProps = {
    setLens: (lens: string) => void, 
    curNode: Node | null
}

export default function SideBarContent({setLens, curNode}: SideBarContentProps) {
    return (
        <>
            <ToggleLensTable setLens={setLens}/>
            <ToggleNodeTable curNode={curNode}/>
        </>
    );
}

