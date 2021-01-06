import React from 'react';
import { Link } from 'react-router-dom';

import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Home from '@material-ui/icons/Home';

import {graplHeaderStyles} from './styles';
import {GraplHeaderProps} from 'types/GraplHeaderTypes'; 

const useStyles = graplHeaderStyles; 

const GraplHeader = (
    {displayBtn}: GraplHeaderProps) => {
        const classes = useStyles();
        
        return(
            <>
                <AppBar position="static">
                    <Toolbar className = {classes.header}>
                        <Typography variant="h6" >
                            GRAPL
                        </Typography>
                        {
                            displayBtn &&
                                <Link to = "/" className = {classes.link}><Home /></Link>
                        }
                    </Toolbar>
                </AppBar>
            </>
        )    
}

export default GraplHeader; 