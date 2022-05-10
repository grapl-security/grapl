import * as React from "react";
import { styled } from "@mui/material/styles";

import Drawer from "@mui/material/Drawer";

import Icon from "@material-ui/core/Icon";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";
import "../../index.css";
import Img from "../../assets/grapl_logo.svg";
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
                    <Icon>
                        <img
                            className={classes.logoImage}
                            src={Img}
                            alt={"Grapl Logo"}
                        />
                    </Icon>
                </div>
            </DrawerHeader>

            <NavListItems></NavListItems>
        </Drawer>
    );
};
