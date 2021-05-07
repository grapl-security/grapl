import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const dasboardStyles = makeStyles((theme: Theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        button: {
            backgroundColor: "#1A191C",
            margin: "1rem",
            fontSize: ".65rem",
        },
        loggedIn: {
            display: "flex",
            justifyContent: "flex-end",
            zIndex: 100,
        },
        dashboard: {
            display: "flex",
            flexDirection: "row",
        },
        link: {
            fontSize: ".75rem",
            color: "white",
            textDecoration: "none",
            padding: ".75rem",
            backgroundColor: "#1A191C",
            margin: "1rem",
            textAlign: "center",
            borderRadius: ".15rem",
            textTransform: "uppercase",
        },
        sagemaker: {
            fontSize: ".65rem",
            color: "white",
            textDecoration: "none",
            padding: ".65rem",
            backgroundColor: "#1A191C",
            margin: "1rem",
            textAlign: "center",
            borderRadius: ".15rem",
            textTransform: "uppercase",
        },
        navSec: {
            margin: "0 auto",
            marginTop: "6rem",
        },
        help: {
            color: "white",
        },
    })
);
