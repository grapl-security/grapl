import React from 'react';
import GraplHeader from "./reusableComponents/GraplHeader";
import { useStyles } from "../modules/uploads/plugins/useStyles";
import { UploadForm } from "../modules/uploads/plugins/uploadPlugins"; 
import { PluginTable } from "../modules/uploads/plugins/pluginTable";

const UploadPlugin = () => {
    const classes = useStyles();
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