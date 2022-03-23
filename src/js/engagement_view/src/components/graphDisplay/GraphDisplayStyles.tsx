import { createStyles, makeStyles } from "@material-ui/core/styles";

const drawerWidth = 500;

export const useStyles = makeStyles((theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        button: {
            backgroundColor: "#42C6FF",
            margin: "0.25rem",
            padding: "0.20rem",
        },
        loggedIn: {
            display: "flex",
            alignItems: "flex-end",
            justifyContent: "flex-end",
        },
        appBar: {
            transition: theme.transitions.create(["margin", "width"], {
                easing: theme.transitions.easing.sharp,
                duration: theme.transitions.duration.leavingScreen,
            }),
        },
        appBarShift: {
            width: `calc(100% - ${drawerWidth}px)`,
            marginLeft: drawerWidth,
            transition: theme.transitions.create(["margin", "width"], {
                easing: theme.transitions.easing.easeOut,
                duration: theme.transitions.duration.enteringScreen,
            }),
        },
        expandLensAndNodeTableIcon: {
            color: "#FFFFFF",
            display: "flex",
            justify: "flex-end",
            fontSize: "6em",
        },
        hide: {
            display: "none",
        },
        drawerPaper: {
            width: drawerWidth,
            flexShrink: 0,
            backgroundColor: theme.palette.background.default,
        },
        drawerHeader: {
            display: "flex",
            alignItems: "center",
            padding: theme.spacing(0, 1),
            // necessary for content to be below app bar
            ...theme.mixins.toolbar,
            justifyContent: "flex-end",
        },
        content: {
            flexGrow: 1,
            padding: theme.spacing(3),
            transition: theme.transitions.create("margin", {
                easing: theme.transitions.easing.sharp,
                duration: theme.transitions.duration.leavingScreen,
            }),
            marginLeft: -drawerWidth,
        },
        contentShift: {
            transition: theme.transitions.create("margin", {
                easing: theme.transitions.easing.easeOut,
                duration: theme.transitions.duration.enteringScreen,
            }),
            marginLeft: 0,
        },
        lensName: {
            margin: "1em",
            color: theme.palette.primary.light,
            fontWeight: "bold",
        },
        headerTitle: {
            fontSize: "35px",
        },
        headerContainer: {
            width: "100vw",
            display: "flex",
            justifyContent: "space-between",
        },
        close: {
            color: "#42C6FF",
        },
        link: {
            color: "#42C6FF",
            textDecoration: "none",
        },
        navIcons: {
            display: "flex",
        },
    })
);
