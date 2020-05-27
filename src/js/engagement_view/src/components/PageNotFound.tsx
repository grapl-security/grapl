import React, { useEffect, useRef } from 'react';
import {createStyles, makeStyles, Theme} from "@material-ui/core/styles";

const useStyles = makeStyles((theme: Theme) =>
    createStyles({
        root: {
            display: "flex",
        },
        button: {
            backgroundColor: "#42C6FF",
            margin: "0.25rem",
            padding: "0.25rem",
        }, 
        errCode: {
            fontSize: "2rem",
            alignContent:"center",
            justifyContent:"center",
            fontColor: "#42C6FF",
        },
        notFound:{
            fontSize: "1.25rem",
            alignContent:"center",
            justifyContent:"center",
            fontColor: "white",
        }, 
    })
);

const PageNotFound = () => {
    const classes = useStyles();
    return(
        <> 
            <h1 className = {classes.errCode}> 404 </h1>
            <h3 className = {classes.notFound}> Page Not Found </h3>        
        </>
    )
}

export default PageNotFound;