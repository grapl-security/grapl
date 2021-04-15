import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const graplHeaderStyles = makeStyles((theme: Theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        header: {
            justifyContent: "space-between",
        },
        link: {
            color: "#42C6FF",
            textDecoration: "none",
        },
    })
);
