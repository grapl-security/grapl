import * as React from "react";

import {Link} from "react-router-dom";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";

import BubbleChartIcon from "@mui/icons-material/BubbleChart";
import PolicyIcon from "@mui/icons-material/Policy";
import ExtensionIcon from "@mui/icons-material/Extension";

import {useStyles} from "../styles/analyzersAndGeneratorsStyles";

export const NavListItems = () => {
    const classes = useStyles();

    return (
        <>
        <List className={classes.drawer}>
            <ListItemButton
                key="Analyzers"
            >
                <ListItemIcon>
                    <PolicyIcon className={classes.icons}/>
                </ListItemIcon>
                <Link to="/analyzers" className={classes.navLink}>
                    {" "}
                    Analyzers{" "}
                </Link>
            </ListItemButton>

            <ListItemButton
                key="Engagements"
            >
                <ListItemIcon>
                    <BubbleChartIcon className={classes.icons}/>
                </ListItemIcon>
                <Link to="/engagements" className={classes.navLink}>
                    {" "}
                    Engagements{" "}
                </Link>
            </ListItemButton>

            <ListItemButton
                key="Generators"
            >
                <ListItemIcon>
                    {<ExtensionIcon className={classes.icons}/>}
                </ListItemIcon>
                <Link to="/generators" className={classes.navLink}>
                    {" "}
                    Generators{" "}
                </Link>
            </ListItemButton>
        </List>

        <List className={classes.drawer}>
            {["Settings", "Logout"].map((text, index) => (
                <ListItemButton
                    key={text}
                >
                    <ListItemIcon className={classes.icons}>
                    </ListItemIcon>
                    <ListItemText primary={text} />
                </ListItemButton>
            ))}
        </List>
        </>
    )
};
