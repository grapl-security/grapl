
import { Field, Form, Formik } from "formik";
import { useStyles } from "../styles/analyzersAndGeneratorsStyles";

const UploadForm = () => {
    const classes = useStyles();
    return (
        <div className={classes.uploadFormContainer}>
            <h3 className={classes.header}> Upload Analyzer</h3>
            <Formik
                initialValues={{ name: "" }} // empty files
                onSubmit={(values, actions) => {
                    setTimeout(() => {
                        // placeholder for now
                        alert(JSON.stringify(values, null, 2));
                        actions.setSubmitting(false);
                    }, 1000);
                }}
            >
                {(props) => (
                    <Form
                        onSubmit={props.handleSubmit}
                        className={classes.uploadForm}
                    >
                        <Field
                            name="displayName"
                            directory=""
                            webkitdirectory=""
                            mozdirectory=""
                            type="text"
                            multiple
                            placeholder="Display Name"
                            onChange={props.handleChange} // in progress
                        />

                        <Field
                            name="plugin"
                            directory=""
                            webkitdirectory=""
                            mozdirectory=""
                            type="file"
                            multiple
                            placeholder="Plugin"
                            onChange={props.handleChange} // in progress
                        />


                        <button type="submit" className={classes.submitBtn}>
                            UPLOAD
                        </button>
                    </Form>
                )}
            </Formik>
        </div>
    );
};


export default UploadForm;