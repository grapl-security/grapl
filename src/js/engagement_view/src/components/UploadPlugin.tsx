import React, {useEffect, useState} from 'react';
import GraplHeader from "./reusableComponents/GraplHeader";
import { useStyles } from "../modules/uploads/plugins/useStyles";
import { UploadForm } from "../modules/uploads/plugins/uploadPlugins"; 
import { PluginTable } from "../modules/uploads/plugins/pluginTable";
import { checkLogin } from '../Login';

const getTimeMod = (mod: number) => {
    const time = Date.now();

    return (time - (time % mod))
}

const UploadPlugin = () => {
    const classes = useStyles();
    const [state, setState] = useState({
        loggedIn: true,
        lastUpdate:  getTimeMod(5000),
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

    return(
        <>
            <GraplHeader displayBtn={true} />
            
            <div className={classes.upload}>
                <div className = {classes.uploadFormContainer}>
                    <UploadForm />
                    <div id = "message" /></div>
                <div className =  {classes.pluginTable}>
                    <PluginTable  />
                </div>
            </div>
        </>
    )
}

export default UploadPlugin;