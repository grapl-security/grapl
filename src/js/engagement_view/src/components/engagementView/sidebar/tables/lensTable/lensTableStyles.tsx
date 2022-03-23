import { makeStyles, createStyles } from "@material-ui/core/styles";

export const useStyles = makeStyles((theme) =>
    createStyles({
        root: {
            fontSize: "1rem",
            border: 0,
            color: "white",
            padding: "0 30px",
        },
        backdrop: {
            color: theme.palette.text.secondary,
            backgroundColor: "transparent",
            width: "80%",
        },
        table: {
            minWidth: 450,
            backgroundColor: theme.palette.background.default,
        },
        lensToggleBtn: {
            width: ".05%",
            height: "50%",
            color: "white",
            margin: ".5rem",
            backgroundColor: theme.palette.background.default,
        },
        title: {
            margin: "1rem",
            fontSize: "1.1rem",
            color: "white",
        },
        expand: {
            color: theme.palette.primary.contrastText,
            margin: "0px",
            width: "1.5rem",
            height: "1.5rem",
        },
        header: {
            display: "flex",
        },
        lensName: {
            fontSize: ".7rem",
        },
        pagination: {
            backgroundColor: theme.palette.background.default,
        },
        tableHead: {
            display: "flex",
            color: "white",
            fontSize: ".8rem",
        },
        tableContainer: {
            textAlign: "center",
            marginLeft: "auto",
            marginRight: "auto",
            width: "95%",
        },
        tableRow: {
            background: theme.palette.background.default,
        },
        lensNameStyle: {
            color: theme.palette.text.primary,
            fontSize: "1rem",
        },
    })
);
