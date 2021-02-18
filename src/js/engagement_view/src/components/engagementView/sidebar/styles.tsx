import { makeStyles } from "@material-ui/core/styles";

export const useStyles = makeStyles({
    root:{
        fontSize: "1rem",
    },
    backdrop: {
        color: "#fff",
        backgroundColor: "transparent",
        width: "80%",
    }, 
    button: {
        width: ".05%",
        height: "50%", 
        color: "white",
        margin: ".5rem",
        backgroundColor:"#424242",
    },
    title: {
        margin: "1rem",
        fontSize: "1.1rem",
        color: "#42C6FF",
    },
    expand:{
        color: "#42C6FF",
        margin: "0px",
        width: "2rem",
        height: "2rem",
    },
    header:{
        display: "flex"
    }, 
    table: {
        minWidth: 450, 
    },
    lensName: {
        fontSize: ".75rem",
    },
    pagination: {
        margin: ".5rem",
        backgroundColor: "#595959",
    },
    head: {
        color: "white",
        fontSize: ".75rem",
    },

});