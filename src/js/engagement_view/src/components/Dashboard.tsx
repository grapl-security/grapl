import React from 'react';
import Button from "@material-ui/core/Button";
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";

const useStyles = makeStyles((theme: Theme) =>
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
        },
        nav: {
            margin: "2rem",
            width: "30%",
            display: "flex",
            flexDirection: "column",
        },
        dashboard:{
            display: "flex",
            flexDirection: "row",
        }, 
    })
);

export default function Dashboard() {
    const classes = useStyles();
    console.log("App loaded");
    return (
        <> 
            <AppBar position="static">
                <Toolbar>
                    <Typography variant="h6" >
                        GRAPL
                    </Typography>
                    <Button className = {classes.button}>Logout</Button>
                </Toolbar>
            </AppBar>

            <div className = { classes.dashboard}>
                <section className = { classes.nav }>
                    <Button className = {classes.button }>Engagements</Button>
                    <Button className = {classes.button }>Upload Plugin</Button>
                </section>

                <section className = { classes.welcome }>
                    <h1>Welcome, username</h1>
                </section>
            </div>
        </>
    )
}
