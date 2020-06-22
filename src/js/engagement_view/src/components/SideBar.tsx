import GraphDisplay from "./GraphViz";
import React from "react";
import SideBarContent from './SideBarContent'
import clsx from "clsx";
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";
import Drawer from "@material-ui/core/Drawer";
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Divider from "@material-ui/core/Divider";
import IconButton from "@material-ui/core/IconButton";
import KeyboardArrowLeftIcon from '@material-ui/icons/KeyboardArrowLeft';
import Button from "@material-ui/core/Button";
import { Node } from "../modules/GraphViz/CustomTypes";
import Home from '@material-ui/icons/Home';

const drawerWidth = 500;

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    root: {
      display: "flex"
    },
    button: {
      backgroundColor: "#42C6FF",
      margin: "0.25rem",
      padding: "0.20rem",
  }, 
    appBar: {
      transition: theme.transitions.create(["margin", "width"], {
        easing: theme.transitions.easing.sharp,
        duration: theme.transitions.duration.leavingScreen
      })
    },
    appBarShift: {
      width: `calc(100% - ${drawerWidth}px)`,
      marginLeft: drawerWidth,
      transition: theme.transitions.create(["margin", "width"], {
        easing: theme.transitions.easing.easeOut,
        duration: theme.transitions.duration.enteringScreen
      })
    },
    menuButton: {
      marginRight: theme.spacing(2),
      color: "#42C6FF"
    },
    hide: {
      display: "none"
    },
    drawer: {
      width: drawerWidth,
      flexShrink: 0
    },
    drawerPaper: {
      width: drawerWidth
    },
    drawerHeader: {
      display: "flex",
      alignItems: "center",
      padding: theme.spacing(0, 1),
      // necessary for content to be below app bar
      ...theme.mixins.toolbar,
      justifyContent: "flex-end"
    },
    content: {
      flexGrow: 1,
      padding: theme.spacing(3),
      transition: theme.transitions.create("margin", {
        easing: theme.transitions.easing.sharp,
        duration: theme.transitions.duration.leavingScreen
      }),
      marginLeft: -drawerWidth
    },
    contentShift: {
      transition: theme.transitions.create("margin", {
        easing: theme.transitions.easing.easeOut,
        duration: theme.transitions.duration.enteringScreen
      }),
      marginLeft: 0,
    },
    lensName:{
      color:"#EAFDFF",
      fontSize: "1.5rem", 
    },
    header:{
      fontSize: "35px",
    }, 
    headerContent: {
      width: "100vw",
      display: "flex",  
      justifyContent: "space-between",
    },
    close:{
      color:"#42C6FF",
    }
  })
);

type SideBarProps = {
  setLens: (lens: string) => void,
  curLens: string,
  curNode: Node | null,
  redirectTo: (pageName: string) => void,
}

export default function SideBar({setLens, curLens, curNode, redirectTo}: SideBarProps) {
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
      {/* <CssBaseline /> */}
      <AppBar
        position="fixed"
        className={
          clsx(classes.appBar,
          { [classes.appBarShift]: open})
        }
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            onClick={handleDrawerOpen}
            edge="start"
            className={clsx(classes.menuButton, open && classes.hide)}
          >
            {/* // Menu Icon  */}
            &#9776;
          </IconButton>

          <div className={classes.headerContent}>
            <Typography 
              variant="h5" 
              noWrap
            >
              <b className={classes.header}> GRAPL </b>
            </Typography>
            <Button 
                className = {classes.button }
                onClick = { (e) => {
                    redirectTo("dashboard");
                } }
            >
                <Home/>
            </Button>
          </div>

        </Toolbar>
      </AppBar>

      <Drawer
        className={classes.drawer}
        variant="persistent"
        anchor="left"
        open={open}
        classes={{
          paper: classes.drawerPaper
        }}
      >
        <div className={classes.drawerHeader}>
          <Button onClick={handleDrawerClose}><KeyboardArrowLeftIcon className={classes.close}/></Button>
        </div>

        <Divider />

        <SideBarContent 
        setLens={setLens} 
        curNode={curNode}
      />

      </Drawer>

      <main
        className={clsx(classes.content, {
          [classes.contentShift]: open
        })}
      >
        <div className={ classes.drawerHeader } />
      <h3 className={ classes.lensName }>
        {/* selected lens name */}
        {curLens || ""} 
      </h3>

        <Typography paragraph></Typography>
      </main>
    </div>
  );
}

type EngagementUxState = {
  curLens: string, 
  curNode: Node | null 
}

const defaultEngagementUxState = (): EngagementUxState => {
  return {
    curLens: "",
    curNode: null,
  }
}

export const EngagementUx = ({redirectTo}: any) => {
    const [state, setState] = React.useState(defaultEngagementUxState());
    
    return (
        <>
            <SideBar 
                setLens={
                    (lens: string) => setState({
                        ...state,
                        curLens: lens,
                    })
                }
                curLens={state.curLens}
                curNode={state.curNode}
                redirectTo={redirectTo}
            />

            <GraphDisplay 
                lensName={state.curLens} 
                setCurNode={(node: Node) => {
                    setState({
                        ...state,
                        curNode: node,
                    })
                }}
            />
        </>
    )
}