import { Link } from "react-router-dom";
import { styled, useTheme } from "@mui/material/styles";

import Drawer from "@mui/material/Drawer";

import List from "@mui/material/List";
import Icon from "@material-ui/core/Icon";

import ListItem from "@mui/material/ListItem";
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
                <ListItem button key="Analyzers">
                    <ListItemIcon>
                        <PolicyIcon className={classes.icons} />
                    </ListItemIcon>
                    <Link to="/analyzers" className={classes.navLink}>
                        {" "}
                        Analyzers{" "}
                    </Link>
                </ListItem>

                <ListItem button key="Engagements">
                    <ListItemIcon>
                        <BubbleChartIcon className={classes.icons} />
                    </ListItemIcon>
                    <Link to="/engagements" className={classes.navLink}>
                        {" "}
                        Engagements{" "}
                    </Link>
                </ListItem>

                <ListItem button key="Generators">
                    <ListItemIcon>
                        {<ExtensionIcon className={classes.icons} />}
                    </ListItemIcon>
                    <Link to="/generators" className={classes.navLink}>
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
                                <LogoutOutlinedIcon className={classes.icons} />
                            )}
                        </ListItemIcon>
                        <ListItemText primary={text} />
                    </ListItem>
                ))}
            </List>
        </Drawer>
    );
};
