import React from "react";

import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableContainer from "@material-ui/core/TableContainer";
import TableRow from "@material-ui/core/TableRow";

import { mapEdgeProps } from "../../graphDisplay/graphLayout/graph_traverse";
import { mapNodeProps } from "../../graphDisplay/graphLayout/mapNodeProps";

import { OtherNodeProperties, VizNode } from "types/CustomTypes";

import { nodeTableHeader } from "./nodeTableHeader";
import { nodeTableStyles } from "./styles";

const useStyles = nodeTableStyles;

type NodeTableProps = {
	node: VizNode;
};

function NodeTable({ node }: NodeTableProps) {
	const classes = useStyles();
	const hidden = new Set([
		"id",
		"dgraph.type",
		"dgraph_type",
		"__indexColor",
		"risks",
		"uid",
		"scope",
		"name",
		"nodeType",
		"nodeLabel",
		"x",
		"y",
		"index",
		"vy",
		"vx",
		"fx",
		"fy",
	]);

	mapEdgeProps(node as any, (edgeName: any, _neighbor: any) => {
		hidden.add(edgeName);
	});

	const displayNode = {} as OtherNodeProperties;

	mapNodeProps(node, (propName: string) => {
		const prop = node[propName];

		if (!hidden.has(propName)) {
			if (prop) {
				if (propName.includes("_time")) {
					try {
						displayNode[propName] = new Date(prop).toLocaleString();
					} catch (e) {
						displayNode[propName] = prop;
					}
				} else {
					displayNode[propName] = prop;
				}
			}
		}
	});

	return (
		<TableContainer>
			<Table className={classes.nodeTable}>
				{nodeTableHeader(node, classes)}
				<TableBody>
					{Object.entries(displayNode).map((nodeProperty) => {
						const [key, value] = nodeProperty;

						return (
							<TableRow key={node.uid + key}>
								<TableCell className={classes.nodeTableData} align="left">
									<b>{key}</b>
								</TableCell>
								<TableCell className={classes.nodeTableData} align="right">
									{value as any}
								</TableCell>
							</TableRow>
						);
					})}
				</TableBody>
			</Table>
		</TableContainer>
	);
}

export default NodeTable;
