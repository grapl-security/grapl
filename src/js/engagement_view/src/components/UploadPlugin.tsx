import React from 'react';
import GraplHeader from "./reusableComponents/GraplHeader";
import { UploadFormProps } from "../modules/uploads/plugins/uploadPluginTypes"
import { useStyles } from "../modules/uploads/plugins/useStyles";
import { UploadForm } from "../modules/uploads/plugins/uploadPlugins"; 
import { PluginTable } from "../modules/uploads/plugins/pluginTable";

const UploadPlugin = ({redirectTo}: UploadFormProps) => {
    const classes = useStyles();
    return(
        <>
            <GraplHeader redirectTo={redirectTo} displayBtn={true} />
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