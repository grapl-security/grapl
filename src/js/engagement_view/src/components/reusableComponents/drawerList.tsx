import * as React from "react";

import { NavLink } from "react-router-dom";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";

import BubbleChartIcon from "@mui/icons-material/BubbleChart";
import PolicyIcon from "@mui/icons-material/Policy";
import ExtensionIcon from "@mui/icons-material/Extension";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";

export const NavListItems = () => {
    const classes = useStyles();

    return (
        <>
            <List className={classes.drawer}>
                <ListItemButton>
                    <NavLink
                        to="/analyzers"
                        className={classes.navLink}
                        style={({ isActive }) => ({
                            color: isActive ? "#1A76D2" : "#C6D1E7",
                        })}
                    >
                        <PolicyIcon className={classes.icons} />
                        Analyzers
                    </NavLink>
                </ListItemButton>

                <ListItemButton key="Engagements">
                    <NavLink
                        to="/"
                        className={classes.navLink}
                        style={({ isActive }) => ({
                            color: isActive ? "#1A76D2" : "#C6D1E7",
                        })}
                    >
                        <BubbleChartIcon className={classes.icons} />
                        Engagements
                    </NavLink>
                </ListItemButton>

                <ListItemButton key="Generators">
                    <NavLink
                        to="/generators"
                        className={classes.navLink}
                        style={({ isActive }) => ({
                            color: isActive ? "#1A76D2" : "#C6D1E7",
                        })}
                    >
                        <ExtensionIcon className={classes.icons} />
                        Generators
                    </NavLink>
                </ListItemButton>
            </List>

            <List className={classes.drawer}>
                {["Settings", "Logout"].map((text, index) => (
                    <ListItemButton key={text}>
                        <ListItemIcon className={classes.icons}></ListItemIcon>
                        <ListItemText primary={text} />
                    </ListItemButton>
                ))}
            </List>
        </>
    );
};
