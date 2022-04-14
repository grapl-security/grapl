import { createStyles, makeStyles } from "@material-ui/core/styles";

const drawerWidth = 500;

export const useStyles = makeStyles((theme) =>
    createStyles({
        root: {
            display: "flex",
            flexDirection: "row",
            alignItems: "flex-end",
            justifyContent: "space-between",
            width: "100vw",
            height: "10vh",
        },
        engagementView: {
            width: "100vw",
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
            flex: "auto",
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
            "&:hover": {
                background: "none",
            },
            color: "#FFFFFF",
            fontSize: "8rem",
            flex: "auto", // will not show up without auto property
            paddingBottom: "1.85rem",
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
        lensNameDisplay: {
            color: theme.palette.primary.light,
            paddingBottom: "1.85rem",
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
