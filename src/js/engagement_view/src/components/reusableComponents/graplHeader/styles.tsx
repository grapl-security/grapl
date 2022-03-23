import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const graplHeaderStyles = makeStyles((theme: Theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        header: {
            justifyContent: "space-between",
        },
        titleIcon: {
            fontSize: "36px", // icons are treated as fonts, weird
        },
        title: {
            flexGrow: 1,
        },
        link: {
            color: theme.palette.background.default,
            textDecoration: "none",
        },
    })
);
