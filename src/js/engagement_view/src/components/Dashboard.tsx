import React, { useEffect, useState } from 'react';
import { checkLogin } from '../Login';
import GraplHeader from "./reusableComponents/GraplHeader";
import { Link } from 'react-router-dom';
import { dasboardStyles } from './makeStyles/DashboardStyles';

const useStyles = dasboardStyles; 

const getTimeMod = (mod: number) => {
    const time = Date.now();

    return (time - (time % mod))
}

export default function Dashboard() {
    const classes = useStyles();
    
    const [state, setState] = useState({
        loggedIn: true,
        lastUpdate: getTimeMod(5000),
    });

    useEffect(() => {
        const now = getTimeMod(5000);

        if (state.lastUpdate !== now) {
            checkLogin()
            .then((loggedIn) => {
                if (!loggedIn) {
                    console.warn("Logged out")
                }
                setState({
                    loggedIn: loggedIn || false,
                    lastUpdate: now
                });
            })
        }
    
    }, [state, setState])

    console.log("state - loggedin", state.loggedIn); 

    if (!state.loggedIn) {
        // Replace with a popup card with a redirect to login 
        window.history.replaceState('#/', "", "#/login");
        window.location.reload();
    }

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
