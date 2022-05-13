import * as React from "react";

import Icon from "@material-ui/core/Icon";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";

import Img from "../../assets/grapl_logo.svg";

export const GraplLogo = () => {
    const classes = useStyles();

    return (
        <Icon>
            <img className={classes.logoImage} src={Img} alt={"Grapl Logo"} />
        </Icon>
    );
};
