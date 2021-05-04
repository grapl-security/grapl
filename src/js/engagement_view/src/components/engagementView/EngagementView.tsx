import React, { useEffect } from "react";
import { Link } from "react-router-dom";
import clsx from "clsx";

import Drawer from "@material-ui/core/Drawer";
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Divider from "@material-ui/core/Divider";
import IconButton from "@material-ui/core/IconButton";
import KeyboardArrowLeftIcon from "@material-ui/icons/KeyboardArrowLeft";
import Button from "@material-ui/core/Button";
import Home from "@material-ui/icons/Home";

import { VizNode } from "types/CustomTypes";
import GraphDisplay from "../graphDisplay/GraphDisplay";
import LensAndNodeTableContainer from "./sidebar/LensAndNodeTableContainer";
import { LoginNotification } from "../reusableComponents";
import { checkLogin } from "../../services/login/checkLoginService";
import { useStyles } from "../graphDisplay/GraphDisplayStyles";

type EngagementViewProps = {
    setLens: (lens: string) => void;
    curLens: string;
    curNode: VizNode | null;
};

const defaultEngagementState = (): EngagementUxState => {
    return { curLens: "", curNode: null, loggedIn: true, renderedOnce: false };
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
        <div className={classes.root}>
            <AppBar
                position="fixed"
                className={clsx(classes.appBar, {
                    [classes.appBarShift]: open,
                })}
            >
                <Toolbar>
                    <IconButton
                        color="inherit"
                        aria-label="open drawer"
                        onClick={handleDrawerOpen}
                        edge="start"
                        className={clsx(
                            classes.menuButton,
                            open && classes.hide
                        )}
                    >
                        {/* // Menu Icon  */}
                        &#9776;
                    </IconButton>

                    <div className={classes.headerContainer}>
                        <Typography variant="h5" noWrap>
                            <b className={classes.headerTitle}> GRAPL </b>
                        </Typography>
                        <Link to="/" className={classes.link}>
                            <Home />
                        </Link>
                    </div>
                </Toolbar>
            </AppBar>

            <Drawer
                className={classes.drawer}
                variant="persistent"
                anchor="left"
                open={open}
                classes={{
                    paper: classes.drawerPaper,
                }}
            >
                <div className={classes.drawerHeader}>
                    <Button onClick={handleDrawerClose}>
                        <KeyboardArrowLeftIcon className={classes.close} />
                    </Button>
                </div>

                <Divider />

                <LensAndNodeTableContainer
                    setLens={setLens}
                    curNode={curNode}
                />
            </Drawer>

            <main
                className={clsx(classes.content, {
                    [classes.contentShift]: open,
                })}
            >
                <div className={classes.drawerHeader} />

                {/* selected lens name */}
                <h3 className={classes.lensName}> {curLens || ""} </h3>

                <Typography paragraph></Typography>
            </main>
        </div>
    );
}

type EngagementUxState = {
    curLens: string;
    curNode: VizNode | null;
    loggedIn: boolean;
    renderedOnce: boolean;
};

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

            <>
                <div className={classes.loggedIn}>
                    {!loggedIn ? <LoginNotification /> : ""}
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
};
