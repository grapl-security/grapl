import React from 'react';
import ReactDOM from 'react-dom';
import GraplHeader from "./reusableComponents/GraplHeader";
import Button from "@material-ui/core/Button";
import {Field, Form, Formik, ErrorMessage} from "formik";

const readFile = (file: any, onRead:any) => {
    const reader = new FileReader();
    reader.addEventListener('load', (event:any) => {
        const fileRes = event.target.result;
        onRead(fileRes);
    });
    reader.readAsDataURL(file);
}

const UploadForm = ({props}: any) => {
    const [curFiles, setFiles] = React.useState([]);
    return(
        <Formik
            initialValues={{ "filename": "" }}
            onSubmit={ (event:any) => { 
                // let file = event.target.files;
                console.log("File", event);
                console.log("files are", curFiles)

                const fileMap = new Map<string,string>();

                for(const f of curFiles){
                    readFile(f, (fileResult:any) => {
                        console.log("fileResult", fileResult);
                        fileMap.set(f, fileResult);

                    })
                }

                const payload = JSON.stringify({payload:fileMap});
                
                
                // if (file) {
                //     let data = new FormData();
                //     data.append('file', file);
                //     // API CALL axios.post('/files', data)...
                // }
            }
            }
        >
            <Form>
                {/* webkitdirectory mozdirectory */}
                {/* accept = property to restrict types, currently we accept ANY type */}
                <Field 
                    className = "inputFiles" 
                    name="plugin" 
                    type="file" 
                    multiple placeholder="Plugin" 
                    onChange={(event: any) => {
                        setFiles(
                            event.currentTarget.files
                        )
                        // console.log(event.currentTarget.files);
                    }}
                /> 
                    
                    <br/>
                <button className="submitBtn"  type="submit">Submit</button>
            </Form>
        </Formik>
    )
}


const UploadPlugin = ({redirectTo}:any) => {
    return(
        <>
            <GraplHeader />
            <UploadForm /> 
        </>
    )
}

export default UploadPlugin;