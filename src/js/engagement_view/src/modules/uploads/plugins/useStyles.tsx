import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const useStyles = makeStyles(
    (theme: Theme) =>
        createStyles({
            root: {
                display: "flex",
            },
            formContainer: {
                color: "white", 
                margin: "1rem",
            },
            btn: {
                backgroundColor: "#42C6FF",
                margin: "0.25rem",
                padding: "0.25rem",
                borderRadius: "6px",
            },
            inputFiles: {
                border: "none",
                width: "35vw",
            },
            upload: {
                display: "flex",
            },
            succMsg: {
                color: "#03dac6",
            }, 
            errMsg: {
                color: "#CF6679",
            }, 
            pluginTable: {
                backgroundColor: "#292929",
                margin: "1em",
                width: "50%",
                height: "100%",
                boxShadow: " 1.5px 1.5px #000000",
            }, 
            uploadFormContainer: {
                backgroundColor: "#292929",
                margin: "1em", 
                width: "50%",
                height: "100%",
                boxShadow: "1.5px 1.5px #000000",
                display: "flex", 
            }, 
            uploadForm: {
                fontFamily: "Arial",
                display: "flex",
                fontSize: ".75rem",
            }
        }
    )
);