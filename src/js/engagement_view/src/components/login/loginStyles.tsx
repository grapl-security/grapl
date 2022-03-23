import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const loginStyles = makeStyles((theme: Theme) =>
    createStyles({
        valErrorMsg: {
            marginLeft: ".8rem",
            color: "#C65454",
            fontSize: ".75rem",
        },
        logoImage: {
            width: "25vw",
            height: "25vh",
        },
        submitBtn: {
            backgroundColor: theme.palette.background.paper,
            color: theme.palette.info.contrastText,
            padding: "1rem",
            borderRadius: "4px",
            width: "100%",
            height: "30%",
            fontWeight: "bold",
            letterSpacing: "2px",
            marginTop: "1em",
            fontSize: ".75em",
        },
        field: {
            backgroundColor: "#2B3648",
        },
        loginContainer: {
            minHeight: "100vh",
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            alignItems: "center",
        },
        loginText: {
            marginBottom: "1em",
            color: theme.palette.secondary.light,
            textAlign: "center",
        },
        formContainer: {
            backgroundColor: theme.palette.background.default,
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            fontSize: "1.25em",
            boxShadow: "1px 1px 2px #313C4E",
            fontWeight: "bold",
            padding: "2em 3em 3em 2em",
            borderRadius: "5px",
        },
    })
);
