import GraphDisplay from "./GraphViz";
import React from "react";
import {useState} from "react";
import SideBarContent from './SideBarContent'
import clsx from "clsx";
import { makeStyles, Theme, createStyles } from "@material-ui/core/styles";
import Drawer from "@material-ui/core/Drawer";
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Divider from "@material-ui/core/Divider";
import IconButton from "@material-ui/core/IconButton";
import KeyboardArrowLeftIcon from '@material-ui/icons/KeyboardArrowLeft';
import Button from "@material-ui/core/Button";

const drawerWidth = 500;

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    root: {
      display: "flex"
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
      fontSize: "2rem", 
      margin: "10px 15px 0px 0px"
    },
    header:{
      fontSize: "35px"
    }, 
    close:{
      color:"#42C6FF",
    }
  })
);

export default function SideBar({setLens, curLens, curNode}: any) {
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
        className={clsx(classes.appBar, {
          [classes.appBarShift]: open
        })}
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
          <Typography 
            variant="h5" 
            noWrap
          >
            <b className={classes.header}> GRAPL</b>
          </Typography>
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







// const darkTheme = createMuiTheme({
//   palette: {
//     type: 'dark',
//     primary: {
//       main: '#73DEFF',
//     }, 
//     secondary: {
//       main: '#81d4fa',
//     },
//   }
// })

// const useStyles = makeStyles({
//   list: {
//     width: 500,
//   },
//   heading: {
//     margin: "1em",
//     color: "#B15DFF"
//   },
//   button:{
//     margin: "1em",
//   }
// });



// export default function SideBar({setLens, curLens, curNode}: any) {
//   const classes = useStyles();
//   const [state, setState] = React.useState({
//     left: false,
//   });

//   const toggleDrawer = (anchor: Anchor, open: boolean) => (
//     event: React.KeyboardEvent | React.MouseEvent
//   ) => {
//     if (
//       event.type === "keydown" &&
//       ((event as React.KeyboardEvent).key === "Tab" ||
//         (event as React.KeyboardEvent).key === "Shift")
//     ) {
//       return;
//     }

//     setState({ ...state, [anchor]: open });
//   };

//   const list = (anchor: Anchor) => (
//     <div
//       className={clsx(classes.list, {})}
//       role="presentation"
//       //#TODO: Make onclick below an X button
//       // onClick={toggleDrawer(anchor, false)}
//       onKeyDown={toggleDrawer(anchor, false)}
//     >
//       <SideBarContent 
//         setLens={setLens} 
//         curNode={curNode}
//       />
//     </div>
//   );

//   return (
//     <>
//       <ThemeProvider theme={darkTheme}>
//       {(["left"] as Anchor[]).map((anchor) => (
//         <React.Fragment key={anchor}>
//           <Button></Button>
//           <Button
//             variant="contained"
//             color="primary"
//             className={classes.button}
//             onClick={toggleDrawer(anchor, true)}
//           >
//             Engagements
//           </Button>

//           <Drawer
//             anchor={anchor}
//             open={state[anchor]}
//             onClose={toggleDrawer(anchor, false)}
//           >
//             {list(anchor)}
//           </Drawer>
          
//           <h3 className = {classes.heading}>{curLens || ""} </h3>
          

//         </React.Fragment>
//       ))}
//     </ThemeProvider>
//     </>
//   );
// }

export const EngagementUx = () => {
    
    const [state, setState] = React.useState({
        curLens: "",
        curNode: null,
    });
    
    console.log('EngagementUX: curLens, ', state.curLens);

    return (
        <>
            <SideBar 
                setLens={
                    (lens: any) => setState({
                        ...state,
                        curLens: lens,
                    })
                }
                curLens={state.curLens}
                curNode={state.curNode}
            />

            <GraphDisplay 
                lensName={state.curLens} 
                setCurNode={(node: any) => {
                    setState({
                        ...state,
                        curNode: node,
                    })
                }}
            />
        </>
    )
}