import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const loginStyles = makeStyles((theme: Theme) =>
    createStyles({
        valErrorMsg: {
            marginLeft: ".8rem",
            color: "red",
            fontSize: ".75rem",
        },
    })
);
