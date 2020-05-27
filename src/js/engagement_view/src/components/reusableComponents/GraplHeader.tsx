import React from 'react';
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Button from "@material-ui/core/Button";
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";

const useStyles = makeStyles((theme: Theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        button: {
            backgroundColor: "#42C6FF",
            margin: "0.25rem",
            padding: "0.20rem",
        }, 
        header: {
            justifyContent: "space-between",
        }
    })
);

const GraplHeader = () => {
    const classes = useStyles();
    return(
        <>
            <AppBar position="static">
                <Toolbar className = {classes.header}>
                    <Typography variant="h6" >
                        GRAPL
                    </Typography>
                    <Button 
                        className = {classes.button}
                        // OnClick Remove Local Storage (Ask Colin)
                    >Logout</Button>
                </Toolbar>
            </AppBar>
        </>
    )    
}

export default GraplHeader; 