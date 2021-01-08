import { makeStyles } from "@material-ui/core/styles";

export const useStyles = makeStyles({
    root:{
        fontSize: "1rem",
    },
    button: {
        width: ".005%",
        color: "white",
        backgroundColor:"#424242",
    },
    title: {
        fontSize: "25px",
        color: "#ffffff",
    },
    icon:{
        color: "#42C6FF",
        margin: "15px 0 0 10px",
    }, 
    expand:{
        color: "#42C6FF",
        margin: "0px"
    },
    header:{
        display: "flex"
    }, 
    table: {
        minWidth: 450, 
    },
    lensName: {
        fontSize: "16px",
    },
    pagination: {
        margin: ".5rem",
        backgroundColor: "#595959",
    }
});