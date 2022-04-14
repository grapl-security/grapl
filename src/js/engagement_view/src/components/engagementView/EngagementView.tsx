import React, {useEffect} from "react";
import clsx from "clsx";

import Drawer from "@material-ui/core/Drawer";
import Divider from "@material-ui/core/Divider";
import IconButton from "@material-ui/core/IconButton";
import KeyboardArrowLeftIcon from "@material-ui/icons/KeyboardArrowLeft";
import Button from "@material-ui/core/Button";
import ManageSearchIcon from "@mui/icons-material/ManageSearch";

import {VizNode, LensName} from "types/CustomTypes";

import GraphDisplay from "../graphDisplay/GraphDisplay";
import LensAndNodeTableContainer from "./sidebar/LensAndNodeTableContainer";
import {LoginNotification} from "../reusableComponents";
import {checkLogin} from "../../services/login/checkLoginService";

import {useStyles} from "../graphDisplay/GraphDisplayStyles";
import CollapsibleNavDrawer from "../reusableComponents/collapsibleDrawer";

import AppBar from '@mui/material/AppBar';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';


type EngagementViewProps = {
    setLens: (lens: string) => void;
    curLens: string;
    curNode: VizNode | null;
};

const defaultEngagementState = (): EngagementUxState => {
    return {curLens: "", curNode: null, loggedIn: true, renderedOnce: false};
};

type EngagementUxState = {
    curLens: string;
    curNode: VizNode | null;
    loggedIn: boolean;
    renderedOnce: boolean;
};

export default function EngagementView({
                                           setLens,
                                           curLens,
                                           curNode,
                                       }: EngagementViewProps) {
    const classes = useStyles();

    const [open, setOpen] = React.useState(true);

    const handleDrawerOpen = () => {
        setOpen(true);
    };

    const handleDrawerClose = () => {
        setOpen(false);
    };

    return (
        <>
            <Box sx={{flexGrow: 1}}>
                <AppBar position="static">
                    <Toolbar sx={ {backgroundColor: "#2B3648", display: "flex"}}>
                        <div className={classes.root}>
                            <CollapsibleNavDrawer/>
                            <Typography variant="h6" component="div" className={classes.lensNameDisplay} sx={{flexGrow: 1}}>
                                {curLens || ""}
                            </Typography>
                            <IconButton
                                aria-label="open drawer"
                                onClick={handleDrawerOpen}
                                edge="end"
                                className={clsx(
                                    classes.expandLensAndNodeTableIcon,
                                    open && classes.hide
                                )}
                            >
                                <ManageSearchIcon  sx={{hover: "none"}}/>
                            </IconButton>

                            <Drawer
                                variant="persistent"
                                anchor="right"
                                open={open}
                                classes={{
                                    paper: classes.drawerPaper,
                                }}
                            >
                                <div className={classes.drawerHeader}>
                                    <Button onClick={handleDrawerClose}>
                                        <KeyboardArrowLeftIcon className={classes.close}/>
                                    </Button>
                                </div>

                                <Divider/>

                                <LensAndNodeTableContainer
                                    setLens={setLens}
                                    curNode={curNode}
                                />
                            </Drawer>
                        </div>
                    </Toolbar>
                </AppBar>
            </Box>
        </>
    )
}

export const EngagementUx = () => {
        const classes = useStyles();

        const [engagementState, setEngagementState] = React.useState(
            defaultEngagementState()
        );

        useEffect(() => {
            if (engagementState.renderedOnce) {
                return;
            }

            const fetchLoginAndSetState = async () => {
                const loggedIn = await checkLogin();
                if (!loggedIn) {
                    console.warn("Logged out");
                }
                setEngagementState({
                    ...engagementState,
                    loggedIn: loggedIn || false,
                    renderedOnce: true,
                });
            };

            // Do the initial fetch, and schedule it to re-run every N seconds
            fetchLoginAndSetState();
            const interval = setInterval(fetchLoginAndSetState, 1000);

            return () => {
                clearInterval(interval);
            };
        }, [engagementState, setEngagementState]);

        const loggedIn = engagementState.loggedIn;

        return (
            <>
                <div className={classes.navIcons}>

                    <EngagementView
                        setLens={(lens: string) =>
                            setEngagementState({
                                ...engagementState,
                                curLens: lens,
                            })
                        }
                        curLens={engagementState.curLens}
                        curNode={engagementState.curNode}
                    />
                </div>

                <>
                    <div className={classes.loggedIn}>
                        {!loggedIn ? <LoginNotification/> : ""}
                    </div>

                    <GraphDisplay
                        lensName={engagementState.curLens}
                        setCurNode={(node) => {
                            setEngagementState({
                                ...engagementState,
                                curNode: node,
                            });
                        }}
                    />
                </>
            </>
        );
    }
;
