import React from "react";

import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';

import { mapEdgeProps } from '../../graphViz/utils/graph/graph_traverse';
import { mapNodeProps } from '../../graphViz/GraphViz';
import { Node } from "../../graphViz/utils/GraphVizCustomTypes";

import { nodeTableStyles } from "./styles"; 

const useStyles = nodeTableStyles; 

function innerTable(node: Node, styles: any) {
    if(node) {
        return (
        <TableHead >
            <TableRow>
                <TableCell 
                    align="left" 
                    className={styles.tableHeader}>
                    <b> PROPERTY </b>
                </TableCell>
                <TableCell 
                    align="left"
                    className={styles.tableHeader}
                >
                    <b> VALUE </b>
                </TableCell>
            </TableRow>
        </TableHead>
    )
    } else {
        return <></>
    }
}

type NodeTableProps = {
    node: Node
}

function NodeTable({node}: NodeTableProps){
    const classes = useStyles();
    const hidden = new Set(
        ['id', 'dgraph.type', 'dgraph_type', '__indexColor', 'risks','uid', 'scope', 'name', 'nodeType', 'nodeLabel', 'x', 'y', 'index', 'vy', 'vx', 'fx', 'fy']
    );

    mapEdgeProps(node, (edgeName: string, _neighbor: Node) => {
        hidden.add(edgeName)
    });

    const displayNode = {} as any;

    mapNodeProps(
        node, 
        (propName: string) => {
            const prop = (node as any)[propName];

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
            <Table className={classes.nodeTable}>
            {
                innerTable(node, classes)
            }
            <TableBody>
                    {
                        Object.entries(displayNode).map((entry) => {
                            const [key, value] = entry;
                            
                            return(
                                <TableRow key = {node.node_key + key}> 
                                    <TableCell className = {classes.nodeTableData} align="left"><b>{key}</b></TableCell>
                                    <TableCell className = {classes.nodeTableData} align="left">{value as any}</TableCell>
                                </TableRow>
                            ) 
                        })
                    }
            </TableBody>
            </Table>
        </TableContainer>
    ) 
}


export default  NodeTable 