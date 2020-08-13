import React from 'react';
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";
import Home from '@material-ui/icons/Home';
import { Link } from 'react-router-dom';

const useStyles = makeStyles(
    (theme: Theme) =>
        createStyles({
            root: {
                display: "flex",
            },
            header: {
                justifyContent: "space-between",
            }, 
            link:{
                color:"#42C6FF", 
                textDecoration: "none"
            }
        }
    )
);

type GraplHeaderProps = {
    displayBtn: boolean
}

const GraplHeader = ({displayBtn}: GraplHeaderProps) => {
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