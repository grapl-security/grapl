import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";

export const dasboardStyles = makeStyles( (theme: Theme) =>
createStyles({
    root: {
        display: "flex",
    },
    button: {
        backgroundColor: "#42C6FF",
        margin: "0.25rem",
        padding: "0.25rem",
    }, 
    welcome: {
        width:"70%",
        textAlign:"center",
        backgroundColor: "#373740",
        height: "100vh",
        color: "white"
    },
    nav: {
        margin: "2rem",
        width: "30%",
        display: "flex",
        flexDirection: "column",
    },
    dashboard: {
        display: "flex",
        flexDirection: "row",
    }, 
    link: {
        color: "white",
        textDecoration: "none",
        padding: ".75rem",
        backgroundColor: "#42C6FF",
        margin: "1rem",
        textAlign: "center",
        borderRadius: ".35rem",
        textTransform: "uppercase",
        fontWeight: "bolder"
    },
    
})
);