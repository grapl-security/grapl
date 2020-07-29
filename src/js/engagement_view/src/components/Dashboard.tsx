import React from 'react';
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

export default function Dashboard() {
        const classes = useStyles();
        return (
            <> 
                <GraplHeader displayBtn={false} />

                <div className = { classes.dashboard}>
                    <section className = { classes.nav }>
                            <Link to = "/engagements" className = {classes.link}> Engagements </Link>
                            <Link to = "/plugins" className = {classes.link}> Upload Plugin </Link>
                    </section>

                    <section className = { classes.welcome }>
                        <h1> Welcome! </h1>
                    </section>
                </div>
            </>
        )
}
