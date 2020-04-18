import React from "react";
import { makeStyles } from '@material-ui/core/styles';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import { mapEdgeProps } from './SideBarContent';
import {mapNodeProps} from './GraphViz';


const useStyles = makeStyles({
    root: {
        fontSize: "1em"
    },
    table: {
        minWidth: 450,
    }, 
    tableHeader: {
        fontSize: "18px", 
        color: "#EAFDFF", 
    }
});

function innerTable(node: any, classes: any) {
    if(node) {
        return (
        <TableHead >
            <TableRow>
                <TableCell 
                    align="left" 
                    className={classes.tableHeader}>
                    <b> PROPERTY </b>
                </TableCell>
                <TableCell 
                    align="left"
                    className={classes.tableHeader}
                >
                    <b> VALUE </b>
                </TableCell>
            </TableRow>
        </TableHead>
    )
    } else {
        return <div></div>
    }
}

function NodeTable({node}:any){
    const classes = useStyles();
    const hidden = new Set(
        ['id', 'dgraph.type', '__indexColor', 'risks','uid', 'scope', 'name', 'nodeType', 'nodeLabel', 'x', 'y', 'index', 'vy', 'vx', 'fx', 'fy']
    );

    mapEdgeProps(node, (edgeName: string, _neighbor: any) => {
        hidden.add(edgeName)
    });

    const displayNode = {} as any; 

    mapNodeProps(
        node, 
        (propName: string) => {
            const prop = node[propName];

            if(!hidden.has(propName)){
                if (prop) {
                    if (propName.includes('_time')) {
                        displayNode[propName] = new Date(prop).toLocaleString()
                    } else {
                        displayNode[propName] = prop;
                    }
                }
            }           
        }
    )

        return(
            <TableContainer>
                <Table className={classes.table}>
                {
                    innerTable(node, classes)
                }
                <TableBody>
                        {
                            Object.entries(displayNode).map((entry) => {
                                const [key, value] = entry;
                                
                                return(
                                    <TableRow>
                                        <TableCell align="left"><b>{key}</b></TableCell>
                                        <TableCell align="left">{value as any}</TableCell>
                                    </TableRow>
                                ) 
                            })
                        }
                </TableBody>
                </Table>
            </TableContainer>
        ) || "no no"
        
    
}


export default  NodeTable 