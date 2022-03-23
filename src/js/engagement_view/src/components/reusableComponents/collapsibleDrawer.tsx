import * as React from "react";
import { Link } from "react-router-dom";
import { styled, useTheme } from "@mui/material/styles";
import Box from "@mui/material/Box";
import Drawer from "@mui/material/Drawer";
import List from "@mui/material/List";

import Divider from "@mui/material/Divider";
import IconButton from "@mui/material/IconButton";
import MenuIcon from "@mui/icons-material/Menu";
import ChevronLeftIcon from "@mui/icons-material/ChevronLeft";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";

import BubbleChartIcon from "@mui/icons-material/BubbleChart";
import PolicyIcon from "@mui/icons-material/Policy";
import LogoutOutlinedIcon from "@mui/icons-material/LogoutOutlined";
import SettingsIcon from "@mui/icons-material/Settings";
import ExtensionIcon from "@mui/icons-material/Extension";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";
import "../../index.css";
import Img from "../../assets/grapl_logo.svg";
import Icon from "@material-ui/core/Icon";

const drawerWidth = 300;

const DrawerHeader = styled("div")(({ theme }) => ({
    display: "flex",
    alignItems: "center",
    margin: "1 em",
    backgroundColor: "#212936",
    color: "#FFFFFF",
}));

const Main = styled("main", { shouldForwardProp: (prop) => prop !== "open" })<{
    open?: boolean;
}>(({ theme, open }) => ({
    flexGrow: 1,
    padding: theme.spacing(3),
    transition: theme.transitions.create("margin", {
        easing: theme.transitions.easing.sharp,
        duration: theme.transitions.duration.leavingScreen,
    }),
    marginLeft: `-${drawerWidth}px`,
    ...(open && {
        transition: theme.transitions.create("margin", {
            easing: theme.transitions.easing.easeOut,
            duration: theme.transitions.duration.enteringScreen,
        }),
        marginLeft: 0,
    }),
}));

export default function CollapsibleNavDrawer() {
    const theme = useTheme();
    const classes = useStyles();
    const [open, setOpen] = React.useState(false);

    const handleDrawerOpen = () => {
        setOpen(true);
    };

    const handleDrawerClose = () => {
        setOpen(false);
    };

    return (
        <Box sx={{ display: "flex" }}>
            <IconButton
                color="inherit"
                aria-label="open drawer"
                onClick={handleDrawerOpen}
                edge="start"
                sx={{ mr: 2, ...(open && { display: "none" }) }}
            >
                <MenuIcon className={classes.navBarOpenCloseIcons} />
            </IconButton>
            <Drawer
                sx={{
                    width: drawerWidth,
                    flexShrink: 0,
                    "& .MuiDrawer-paper": {
                        width: drawerWidth,
                        boxSizing: "border-box",
                    },
                }}
                variant="persistent"
                anchor="left"
                open={open}
            >
                <DrawerHeader>
                    <div>
                        <IconButton onClick={handleDrawerClose}>
                            {theme.direction === "ltr" ? (
                                <ChevronLeftIcon className={classes.icons} />
                            ) : (
                                <ChevronRightIcon />
                            )}
                        </IconButton>
                        <Icon>
                            <img className={classes.logoImage} src={Img} />
                        </Icon>
                    </div>
                </DrawerHeader>
                <Divider />
                <List className={classes.drawer}>
                    <ListItem button key="Analyzers">
                        <ListItemIcon>
                            <PolicyIcon className={classes.icons} />
                        </ListItemIcon>
                        <Link
                            to="/analyzers"
                            style={{ textDecoration: "none" }}
                        >
                            {" "}
                            Analyzers{" "}
                        </Link>
                    </ListItem>

                    <ListItem button key="Engagements">
                        <ListItemIcon>
                            <BubbleChartIcon className={classes.icons} />
                        </ListItemIcon>
                        <Link
                            to="/engagements"
                            style={{ textDecoration: "none" }}
                        >
                            {" "}
                            Engagements{" "}
                        </Link>
                    </ListItem>

                    <ListItem button key="Generators">
                        <ListItemIcon>
                            {<ExtensionIcon className={classes.icons} />}
                        </ListItemIcon>
                        <Link
                            to="/generators"
                            style={{ textDecoration: "none" }}
                        >
                            {" "}
                            Generators{" "}
                        </Link>
                    </ListItem>
                </List>

                <List className={classes.drawer}>
                    {["Settings", "Logout"].map((text, index) => (
                        <ListItem button key={text}>
                            <ListItemIcon className={classes.icons}>
                                {index % 2 === 0 ? (
                                    <SettingsIcon className={classes.icons} />
                                ) : (
                                    <LogoutOutlinedIcon
                                        className={classes.icons}
                                    />
                                )}
                            </ListItemIcon>
                            <ListItemText primary={text} />
                        </ListItem>
                    ))}
                </List>
            </Drawer>
            <Main open={open}>
                <DrawerHeader />
            </Main>
        </Box>
    );
}
