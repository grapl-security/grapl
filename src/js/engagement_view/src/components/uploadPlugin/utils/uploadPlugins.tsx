import React, { SyntheticEvent } from 'react';
import { Field, Form, Formik } from "formik";

import Button from "@material-ui/core/Button";
import CloudUploadIcon from '@material-ui/icons/CloudUpload';

import { Event, UploadFormState, DirectoryUpload, MessageProps} from "../../../types/uploadPluginTypes"
import { uploadFilesToDgraph } from "../../../services/uploadPlugin/uploadFilesToDgraph";
import { useStyles } from "../uploadPluginStyles";

const readFile = async (file: Blob): Promise <string> => {
    return await new Promise(
        (resolve, reject) => {
            const reader = new FileReader();
            reader.addEventListener('load', (event: ProgressEvent<FileReader>) => {
                if (event.target === null) {
                    console.warn("onload event was null");
                    return;
                } else {
                    const fileRes = event.target.result;
                    if (fileRes === null) {
                        console.warn("Read pluginfile was null");
                        reject(null)
                        return;
                    } else {
                        if (typeof(fileRes) === 'string') {
                            resolve(fileRes);
                        } else {
                            resolve(new TextDecoder().decode(fileRes));
                        }
                    }
                }
            });
            return reader.readAsText(file);
        }
    )
}

const defaultUploadFormState = (): UploadFormState => {
    return {curFiles: null, success: null}
}

const Message = ({status}: MessageProps) => {
    const classes = useStyles();
    if (status === true) {
        return(
            <div className = {classes.succMsg}>Plugins Successfully Deployed!</div>
        )
    } else if (status === false){
        return(
            <div className = {classes.errMsg}>Upload Unsuccessful</div>
        )
    } else {
        return (<div />)
    }
    
}

export const UploadForm = () => {
    const [state, setState] = React.useState(defaultUploadFormState());
    const classes = useStyles();

    return(
        <div className = {classes.formContainer}>
            <h4>UPLOAD DIRECTORY WITH PLUGINS:</h4>
            <Formik
                initialValues={{ "filename": "" }}
                onSubmit={
                    (event: Event) => { 
                        const fileMap = {} as any;
                        const reads: Promise<void>[] = [];
                        const {curFiles} = state;

                        if (curFiles === null) {
                            console.warn("Attempted to upload files without selecting any");
                            return
                        }
                        
                        for(const f of curFiles){
                            let fileRead = readFile(f).then((fileResult) => {
                                // This any is because webkitRelativePath is not standard, and therefor
                                // is not part of the File type
                                const path = ((f as any).webkitRelativePath || (f as any).mozRelativePath);

                                fileMap[path] = fileResult;
                            });
                            reads.push(fileRead);
                        }
                        let success = true;
                        Promise.all(reads).then(() => {
                            return uploadFilesToDgraph({plugins: fileMap});
                        })
                        .then((didSucceed) => {success = success && didSucceed;})
                        .then(() => setState({...state, success}))
                    }
                }
            >
                <Form className = {classes.uploadForm}>
                    {/* accept = property to restrict types, currently we accept ANY type */}
                    <Field 
                        className = {classes.inputFiles}
                        name="plugin" 
                        directory="" 
                        webkitdirectory=""
                        mozdirectory=""
                        type="file" 
                        multiple placeholder="Plugin" 
                        onChange={
                            (event: SyntheticEvent<DirectoryUpload>) => {
                                setState({
                                    ...state,
                                    curFiles: event.currentTarget.files
                                })
                            }
                        }
                    /> 
                    <Button className="submitBtn"  type="submit"><CloudUploadIcon className = {classes.btn}/></Button>
                </Form>
            </Formik>
            <br />
            <Message status = {state.success}/>
        </div>
    )
}