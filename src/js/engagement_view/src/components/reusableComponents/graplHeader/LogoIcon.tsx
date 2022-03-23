import { SvgIcon } from "@material-ui/core";
import { ReactComponent as Logo } from "../../../assets/grapl_logo.svg";

import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

export const LogoIconStyle = makeStyles((theme: Theme) =>
    createStyles({
        logoIcon: {
            marginRight: "0.5em",
            color: "transparent",
        },
    })
);

interface LogoIconProps {
    className: string;
}

export function LogoIcon(props: LogoIconProps) {
    const classes = LogoIconStyle();
    return (
        <SvgIcon
            component={Logo}
            viewBox="150 0 400 500"
            className={classes.logoIcon}
        ></SvgIcon>
    );
}
