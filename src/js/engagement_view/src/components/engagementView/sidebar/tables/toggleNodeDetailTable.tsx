import React, { useState } from "react";

import Button from "@material-ui/core/Button";
import KeyboardArrowDownOutlinedIcon from "@material-ui/icons/KeyboardArrowDownOutlined";
import KeyboardArrowUpOutlinedIcon from "@material-ui/icons/KeyboardArrowUpOutlined";
import { ToggleNodeTableProps } from "types/LensAndNodeTableTypes";

import { NodeDetails } from "../LensAndNodeTableContainer";

import { useStyles } from "./lensTable/lensTableStyles";


export function ToggleNodeDetailTable({ curNode }: ToggleNodeTableProps) {
	const [toggled, setToggle] = useState(true);
	const classes = useStyles();
	return (
		<div>
			{curNode && (
				<div className={classes.header}>
					<b className={classes.title}> Node Details</b>
					<Button
						className={classes.lensToggleBtn}
						onClick={() => {
							setToggle((toggled) => !toggled);
						}}
					>
						{toggled === true ? (
							<KeyboardArrowUpOutlinedIcon className={classes.expand} />
						) : (
							<KeyboardArrowDownOutlinedIcon className={classes.expand} />
						)}
					</Button>
				</div>
			)}

			<div className="nodeToggle">
				{toggled && curNode && <div>{<NodeDetails node={curNode} />}</div>}
			</div>
		</div>
	);
}
