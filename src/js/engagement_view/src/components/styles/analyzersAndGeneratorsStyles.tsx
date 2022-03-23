import { createStyles, makeStyles } from "@material-ui/core/styles";

export const useStyles = makeStyles((theme) =>
    createStyles({
        root: {
            display: "flex",
            color: theme.palette.text.primary,
            border: "none",
        },
        header: {
            color: theme.palette.secondary.light,
            margin: ".5em",
        },
        icons: {
            color: theme.palette.primary.contrastText,
        },
        navBarOpenCloseIcons: {
            color: theme.palette.info.contrastText,
            margin: "1em",
        },
        drawer: {
            backgroundColor: theme.palette.background.default,
            color: theme.palette.text.primary,
            height: "95vh",
        },
        logoImage: {
            display: "flex",
            marginLeft: ".5em",
            height: "200px",
            width: "200px",
        },
        metricsAndUploadContainer: {
            margin: ".6rem",
        },
        metricContainer: {
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            width: 156,
            height: 156,
        },
        metricsIcon: {
            textAlign: "center",
            marginLeft: ".65rem",
            marginTop: "1.25rem",
        },
        metricsText: {
            textAlign: "center",
        },
        uploadFormContainer: {
            width: "35vw",
            height: "30vh",
            margin: "1rem",
            padding: "5rem",
            backgroundColor: theme.palette.background.default,
            borderRadius: "4px",
        },
        uploadForm: {
            display: "flex",
            border: "none",
        },
        submitBtn: {
            backgroundColor: theme.palette.background.paper,
            color: theme.palette.info.contrastText,
            padding: ".7rem",
            marginTop: ".9em",
            marginLeft: "1em",
            borderRadius: "4px",
            width: "30%",
            height: "100%",
            fontFamily: "Roboto",
            fontWeight: "bold",
            letterSpacing: "2px",
        },
        generatorsListTable: {
            height: "95vh",
            width: "30vw",
            color: theme.palette.text.primary,
            padding: "1rem",
        },
        tableRow: {
            color: theme.palette.info.contrastText,
        },
    })
);

// Note: For the following elements, we're styling inline on the component itself to override default styles
// DataGrid (Generator Table Cells)
// Paper
