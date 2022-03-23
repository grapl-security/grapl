import { makeStyles, createStyles } from "@material-ui/core/styles";

export const notificationsStyles = makeStyles((theme) =>
    createStyles({
        root: {
            maxWidth: 345,
            postion: "fixed",
        },
        button: {
            border: "2px solid white",
            backgroundColor: theme.palette.background.default,
        },
    })
);
