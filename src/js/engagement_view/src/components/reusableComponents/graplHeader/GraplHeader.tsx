import React from "react";
import { Link } from "react-router-dom";

import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Home from "@material-ui/icons/Home";

import { graplHeaderStyles } from "./styles";
import { GraplHeaderProps } from "types/GraplHeaderTypes";

import { LogoIcon } from "./LogoIcon";
import { IconButton } from "@material-ui/core";

const useStyles = graplHeaderStyles;

const GraplHeader = ({ displayBtn }: GraplHeaderProps) => {
    const classes = useStyles();

    return (
        <>
            <AppBar position="static">
                <Toolbar className={classes.header}>
                    <LogoIcon className={classes.titleIcon} />
                    <Typography variant="h6" className={classes.title}>
                        GRAPL
                    </Typography>
                    {displayBtn && (
                        <Link to="/" className={classes.link}></Link>
                    )}
                </Toolbar>
            </AppBar>
        </>
    );
};

export default GraplHeader;
