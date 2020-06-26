import React from 'react';
import AppBar from "@material-ui/core/AppBar";
import Toolbar from "@material-ui/core/Toolbar";
import Typography from "@material-ui/core/Typography";
import Button from "@material-ui/core/Button";
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";
import Home from '@material-ui/icons/Home';
import {Link} from 'react-router-dom';

const useStyles = makeStyles(
    (theme: Theme) =>
        createStyles({
            root: {
                display: "flex",
            },
            button: {
                backgroundColor: "#42C6FF",
                margin: "0.25rem",
                padding: "0.20rem",
            }, 
            header: {
                justifyContent: "space-between",
            }, 
            link:{
                color:"white", 
                textDecoration: "none"
            }
        }
    )
);
// history.replaceState 
// function logout() {
//     localStorage.removeItem("grapl_curPage");
//     window.location.reload(false);
// }

type GraplHeaderProps = {
    redirectTo: (pageName: string) => void,
    displayBtn: boolean
}

const GraplHeader = ({redirectTo, displayBtn}: GraplHeaderProps) => {
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
                            <Button 
                            className = {classes.button }
                            >
                                <Link to = "/" className = {classes.link}><Home /></Link>
                            </Button>
                    }
                </Toolbar>
            </AppBar>
        </>
    )    
}

export default GraplHeader; 