import React from "react";

import Button from "@material-ui/core/Button";
import TableCell from "@material-ui/core/TableCell";
import TableRow from "@material-ui/core/TableRow";

import { useStyles } from '../styles';

import { SelectLensProps } from "types/LensAndNodeTableTypes"


export function SelectLens(props: SelectLensProps) {
	const classes = useStyles();
	
    
	return (
		<TableRow className={classes.tableRow} key={props.uid}>
			<TableCell component="th" scope="row" align="left">
				<Button
					className={classes.lensName}
					onClick={() => {
						props.setLens(props.lens);
					}}
				>
					{props.lens_type + " :\t\t" + props.lens}
				</Button>
			</TableCell>

			<TableCell component="th" scope="row" align="right">
				{props.score}
			</TableCell>
		</TableRow>
	);
}
