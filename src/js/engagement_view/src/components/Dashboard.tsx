import React from 'react';
import Button from "@material-ui/core/Button";
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";
import GraplHeader from "./reusableComponents/GraplHeader";
import {Link} from 'react-router-dom';

const useStyles = makeStyles( (theme: Theme) =>
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
        link:{
            color: "white",
            textDecoration: "none"
        }
    })
);

export default function Dashboard() {
        const classes = useStyles();
        return (
            <> 
                <GraplHeader displayBtn={false} />

                <div className = { classes.dashboard}>
                    <section className = { classes.nav }>
                        
                        <Button  className = {classes.button }>
                            <Link to = "/engagements" className = {classes.link}> Engagements </Link>
                        </Button>
                        
                        <Button  className = {classes.button }>
                            <Link to = "/plugins" className = {classes.link}> Upload Plugin </Link>
                        </Button>
                        
                    </section>

                    <section className = { classes.welcome }>
                        <h1> Welcome, username </h1>
                    </section>
                </div>
            </>
        )
}
