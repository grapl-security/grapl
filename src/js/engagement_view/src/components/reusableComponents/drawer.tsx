import * as React from "react";
import { styled } from "@mui/material/styles";

import Drawer from "@mui/material/Drawer";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";
import "../../index.css";
import { GraplLogo } from "./graplLogo";
import { NavListItems } from "./drawerList";

const drawerWidth = 300;

const DrawerHeader = styled("div")(({ theme }) => ({
    display: "flex",
    alignItems: "center",
    margin: "1 em",
    backgroundColor: "#212936",
    color: "#FFFFFF",
}));

export const NavigationDrawer = () => {
    const classes = useStyles();

    return (
        <Drawer
            sx={{
                width: drawerWidth,
                flexShrink: 0,
                "& .MuiDrawer-paper": {
                    width: drawerWidth,
                    boxSizing: "border-box",
                },
            }}
            variant="permanent"
            anchor="left"
        >
            <DrawerHeader>
                <div>
                    <GraplLogo></GraplLogo>
                </div>
            </DrawerHeader>

            <NavListItems></NavListItems>
        </Drawer>
    );
};
