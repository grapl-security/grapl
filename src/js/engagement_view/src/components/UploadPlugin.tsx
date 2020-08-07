import React, {useEffect, useState} from 'react';
import GraplHeader from "./reusableComponents/GraplHeader";
import { useStyles } from "../modules/uploads/plugins/useStyles";
import { UploadForm } from "../modules/uploads/plugins/uploadPlugins"; 
import { PluginTable } from "../modules/uploads/plugins/pluginTable";
import { checkLogin } from '../Login';
import LoginNotification from "./reusableComponents/Notifications";


const UploadPlugin = () => {
    const classes = useStyles();
    const [state, setState] = useState({
        loggedIn: true,
        renderedOnce:  false,
    });

    useEffect(() => {
        if (state.renderedOnce){
            return
        }
        const interval = setInterval(async() => {
            await checkLogin()
            .then((loggedIn) => {
                if(!loggedIn){
                    console.warn("Logged Out")
                }
                setState({
                    loggedIn: loggedIn || false, 
                    renderedOnce: true,
                });
            });
        }, 2000);
        return () => { clearInterval(interval) } 
    }, [state, setState]);

    const loggedIn = state.loggedIn;
    
    return(
        <>
            <GraplHeader displayBtn={true} />

            <div className = {classes.loggedIn}>
                {!loggedIn ? <LoginNotification /> : ""}
            </div>
        
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