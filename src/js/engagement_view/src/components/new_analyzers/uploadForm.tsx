import {Form, Formik} from "formik";
import {useStyles} from "../styles/analyzersAndGeneratorsStyles";
import { createPluginService } from "services/uploadPlugin/createPlugin";

import * as Yup from "yup";


import {Button} from '@mui/material';
import Box from '@mui/material/Box';




const UploadForm = () => {
    const classes = useStyles();

    const analyzerValidationSchema = Yup.object().shape({
        userName: Yup.string().required("Analyzer Name Required"),
        password: Yup.string().required("File Required"),
    });

    return (
        <div className={classes.uploadFormContainer}>
            <h3 className={classes.header}> Upload Analyzer</h3>

            <Formik
                initialValues={{filename: "", analyzerName: ""}}
                validationSchema={analyzerValidationSchema}

                onSubmit={
                    async (values) => {
                        console.log("values", values)
                        const uploadSuccess = await createPluginService(
                            values.filename,
                            values.analyzerName
                        );

                        console.log("uploadSuccess",uploadSuccess)

                        // if(uploadSuccess === true){
                        //     console.log("Uploded")
                        // } else {
                        //     console.error("problem uploading")
                        // }
                        // values.profile.forEach((photo: any, index: any) => {
                        //     data.append(`photo$`, values.profile[index]);
                        // });

                        // axios
                        //     .post("you_api_for_file_upload", data, {
                        //         headers: {
                        //             "Content-Type": "multipart/form-data",
                        //         },
                        //     })
                        //     .then((response) => {
                        //         console.log(response);
                        //     })
                        //     .catch((err) => {
                        //         console.log(err);
                        //     });
                    }
                }
            >

                {(formik: any) => {
                    return (
                    <>
                    <Form>
                        <input
                            id="analyzerName"
                            name="analyzer"
                            type="text"
                            placeholder="Analyzer Name"
                        />

                        <input
                        id="filename"
                        name="filename"
                        type="file"
                        onChange={(event) => {
                        const files = event.target.files;
                        // let myFiles =Array.from(files);
                        // formik.setFieldValue("profile", myFiles);
                    }}
                        multiple
                        />
                        <Button type="submit">Submit</Button>
                    </Form>
                    </>
                    );
                }}
        </Formik>
</div>

)
    ;
};


export default UploadForm;