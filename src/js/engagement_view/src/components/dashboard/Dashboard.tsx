import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import Button from "@material-ui/core/Button";

import GraplHeader from "../reusableComponents/graplHeader/GraplHeader";
import LoginNotification from "../reusableComponents/notifications/Notifications";

import { checkLogin } from '../../services/loginService';
import { getNotebookUrl } from "../../services/notebookService";

import { dasboardStyles } from './styles';

const useStyles = dasboardStyles; 

export default function Dashboard() {
    const classes = useStyles();
    
    const [state, setState] = useState({
        loggedIn: true,
        renderedOnce: false,
    });

    useEffect(
        () => {
            if(state.renderedOnce){
                return;
            }

            const interval = setInterval(async () => {
                await checkLogin()
                .then((loggedIn) => {
                    if(!loggedIn){
                        console.warn("Logged out")
                    }
                    setState({
                        loggedIn: loggedIn || false, 
                        renderedOnce: true,
                    });
                });
            }, 2000);

            return () => clearInterval(interval) 
        }, 
        [state, setState]
    )

    const loggedIn = state.loggedIn;
    console.log("Logged in ", state.loggedIn); 

    return (
        <> 
            <GraplHeader displayBtn={false} />

            <div className = { classes.dashboard}>
                <section className = { classes.nav }>
                        <Link to = "/engagements" className = {classes.link}> Engagements </Link>
                        <Link to = "/plugins" className = {classes.link}> Upload Plugin </Link>
                        <Button onClick={getNotebookUrl} className = {classes.link}> Open Engagement Notebook </Button>
                </section>
                
                <section className = { classes.welcome }>
                    <div className = {classes.loggedIn}>
                        {!loggedIn ? <LoginNotification /> : ""}
                    </div>
                </section>
            </div>
        </>
    )
}       
