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

function SelectLens(props: any) {
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

function ToggleLensTable({setLens}: any) {
    const [state, setState] = useState({
        toggled: true,
        lenses: [],
    });

    const classes = useStyles();

    useEffect(() => {
        const interval = setInterval(() => {
            console.log("Fetching lenses");
            getLenses()
                .then((response) => {
                    console.log('response', response);
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
                        (_lens) => {
                            const lens = _lens as any;
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

const isLocal = true;

const getEngagementEdge = (port?: undefined | string) => {
    if (isLocal) {
        return "http://" + window.location.hostname + (port || ":8900/")
    } else {
        return "__engagement_ux_standin__hostname__"
    }
}

// const engagement_edge = getEngagementEdge();
const graphql_edge = getEngagementEdge(":5000/");


const getLenses = async () => {
    console.log('fetching graph');

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
        .then((res) => res.data);

        console.log(res);
        const jres = await res;

    return jres;
};

export const mapEdgeProps = (node: any, f: any) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                for (const neighbor of node[prop]) {
                    if (neighbor.uid !== undefined) {
                        f(prop, neighbor)
                    }
                }
            }
        }
    }
};

const NodeDetails = ({node}: any) => {
    // #TODO: Remove hidden fields from our node before displaying
    // Display remaining fields of node in our component="div"

    return (
        <>
            <NodeTable node={node} />
        </>
    )
}


function ToggleNodeTable({curNode}: any) {
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
                    toggled && 
                        <>
                            { <NodeDetails node={curNode}/> }
                        </>
                }
            </div>
        </div>
        </>
    )
}

export default function SideBarContent({setLens, curNode}: any) {
    return (
        <>
            <ToggleLensTable setLens={setLens} />
            <ToggleNodeTable curNode={curNode} />
        </>
    );
}

