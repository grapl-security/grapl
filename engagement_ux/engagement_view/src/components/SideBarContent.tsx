import React from "react";
import { useState, useEffect } from "react";
import NodeTable from './NodeTable'
import Button from "@material-ui/core/Button";
import { makeStyles } from "@material-ui/core/styles";
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import BubbleChartIcon from '@material-ui/icons/BubbleChart';
import LensIcon from '@material-ui/icons/Lens';
import Typography from "@material-ui/core/Typography";
import Divider from "@material-ui/core/Divider";


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
    }
});

function Lens(props: any) {
    // #TODO: Only set the lens on click
    return (
        <>
            <Button 
                onClick={
                    () => { 
                        props.setLens(props.lens)    
                    }
            }>
                {props.lens + props.score}            
            </Button>
        </>
    )
}

function ToggleLensTable({setLens}: any) {
    const [state, setState] = useState({
        toggled: true,
        lenses: []
    });
    const classes = useStyles();
    useEffect(() => {
        getLenses()
        .then((response) => {
            console.log(response.lenses);
            setState({
                ...state,
                lenses: response.lenses || [],
            })
        })
    }, []);

    return (
        <>
            <div className={classes.header}>
                <b className={classes.title}>
                    <BubbleChartIcon className = {classes.icon} />
                    LENSES 
                </b>
                <Button
                    // variant="contained"
                    // color="primary"
                    className = {classes.button}
                    onClick={() => { 
                        setState({
                            ...state,
                            toggled: !state.toggled,
                        }) 
                    }}> <ExpandMoreIcon className={classes.expand}/> 
                </Button>
            </div>

            <div className="lensToggle">
                {state.toggled && state.lenses &&
                    state.lenses.map(
                        (_lens) => {
                            const lens = _lens as any;
                            return <Lens 
                                key={new Number(lens.uid)}
                                uid={lens.uid}
                                lens={lens.lens}
                                score={lens.score}
                                setLens={setLens}
                            />
                        }
                    )
                }
            </div>
            <Divider />
        </>
    )
}

const engagement_edge = "http://localhost:8900/";

const getLenses = async () => {
    const res = await fetch(`${engagement_edge}getLenses`,
        {
            method: 'post',
            body: JSON.stringify({
                'prefix': '',
            }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        });
    const jres = await res.json();

    return jres['success'];
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

