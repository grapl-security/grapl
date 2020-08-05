import GraphDisplay from "./GraphViz";
import React, {useEffect} from "react";
import EngagementViewContent from './EngagementViewContent'
import clsx from "clsx";
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
import { Link } from 'react-router-dom';
import { checkLogin } from '../Login';
import LoginNotification from "./reusableComponents/Notifications";
import { useStyles } from "./makeStyles/GraphVizStyles";

type EngagementViewProps = {
  setLens: (lens: string) => void,
  curLens: string,
  curNode: Node | null
}

const defaultEngagementUxState = (): EngagementUxState => {
  return {
    curLens: "",
    curNode: null,
    loggedIn: true,
    renderedOnce: false
  }
}

export default function EngagementView({setLens, curLens, curNode}: EngagementViewProps) {
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
                <Link to = "/" className = {classes.link}> <Home/> </Link>
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

        <EngagementViewContent 
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
  curNode: Node | null,
  loggedIn: boolean,
  renderedOnce: boolean,
}

export const EngagementUx = () => {
    const classes = useStyles();
    const [state, setState] = React.useState(defaultEngagementUxState());

    useEffect(
      () => {
        if (state.renderedOnce) {
          return;
        }
        const interval = setInterval(async () => {
          await checkLogin()
          .then((loggedIn) => {
              if (!loggedIn) {
                  console.warn("Logged out")
              }
              setState({
                  ...state,
                  loggedIn: loggedIn || false,
                  renderedOnce: true,
              });
          });
        }, 2000);
        return () => { clearInterval(interval) }
      }, 
    [state, setState]);

  const loggedIn = state.loggedIn; 
  
    return (
        <>
            <EngagementView 
                setLens={
                    (lens: string) => setState({
                        ...state,
                        curLens: lens,
                    })
                }
                curLens={state.curLens}
                curNode={state.curNode}
            />
        <>
          <div className = {classes.loggedIn}>
            {!loggedIn ? <LoginNotification /> : ""}
          </div>

          <GraphDisplay 
              lensName={state.curLens} 
              setCurNode={
                (node: Node) => {
                  setState({
                      ...state,
                      curNode: node,
                  })
              }}
          />
        </>
      </>
    )
}