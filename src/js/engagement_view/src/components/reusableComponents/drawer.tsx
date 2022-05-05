import * as React from "react";
import { Link } from "react-router-dom";
import { styled, useTheme } from "@mui/material/styles";

import Drawer from "@mui/material/Drawer";

import List from "@mui/material/List";
import Icon from "@material-ui/core/Icon";

import ListItemButton from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";

// Drawer Icons
import BubbleChartIcon from "@mui/icons-material/BubbleChart";
import PolicyIcon from "@mui/icons-material/Policy";
import LogoutOutlinedIcon from "@mui/icons-material/LogoutOutlined";
import SettingsIcon from "@mui/icons-material/Settings";
import ExtensionIcon from "@mui/icons-material/Extension";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";
import "../../index.css";
import Img from "../../assets/grapl_logo.svg";

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

    const location = window.location.href.split("/").slice().pop();

    let index: number = 0;

    location === "analyzers" ? (index = 0) : (index = 2);

    const [selectedIndex, setSelectedIndex] = React.useState(index);

    const handleListItemClick = (
        event: React.MouseEvent<HTMLDivElement, MouseEvent>,
        index: number
    ) => {
        setSelectedIndex(index);
    };

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
                        <img className={classes.logoImage} src={Img} />
                    </Icon>
                </div>
            </DrawerHeader>

            <List className={classes.drawer}>
                <ListItemButton
                    button
                    key="Analyzers"
                    selected={selectedIndex === 0}
                    onClick={(event) => handleListItemClick(event, 0)}
                >
                    <ListItemIcon>
                        <PolicyIcon className={classes.icons} />
                    </ListItemIcon>
                    <Link to="/analyzers" className={classes.navLink}>
                        {" "}
                        Analyzers{" "}
                    </Link>
                </ListItemButton>

                <ListItemButton
                    button
                    key="Engagements"
                    selected={selectedIndex === 1}
                    onClick={(event) => handleListItemClick(event, 1)}
                >
                    <ListItemIcon>
                        <BubbleChartIcon className={classes.icons} />
                    </ListItemIcon>
                    <Link to="/engagements" className={classes.navLink}>
                        {" "}
                        Engagements{" "}
                    </Link>
                </ListItemButton>

                <ListItemButton
                    button
                    key="Generators"
                    selected={selectedIndex === 2}
                    onClick={(event) => handleListItemClick(event, 2)}
                >
                    <ListItemIcon>
                        {<ExtensionIcon className={classes.icons} />}
                    </ListItemIcon>
                    <Link to="/generators" className={classes.navLink}>
                        {" "}
                        Generators{" "}
                    </Link>
                </ListItemButton>
            </List>

            <List className={classes.drawer}>
                {["Settings", "Logout"].map((text, index) => (
                    <ListItemButton button key={text}>
                        <ListItemIcon className={classes.icons}>
                            {index % 2 === 0 ? (
                                <SettingsIcon className={classes.icons} />
                            ) : (
                                <LogoutOutlinedIcon className={classes.icons} />
                            )}
                        </ListItemIcon>
                        <ListItemText primary={text} />
                    </ListItemButton>
                ))}
            </List>
        </Drawer>
    );
};
