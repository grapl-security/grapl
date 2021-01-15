import React from 'react';

import Card from '@material-ui/core/Card';
import CardActionArea from '@material-ui/core/CardActionArea';
import CardActions from '@material-ui/core/CardActions';
import CardContent from '@material-ui/core/CardContent';
import Button from '@material-ui/core/Button';
import Typography from '@material-ui/core/Typography';

import { notificationsStyles } from './styles';

const useStyles = notificationsStyles; 

export default function LoginNotification() {
    const classes = useStyles();

    return (
        <Card variant="outlined" className={classes.root}>
            <CardActionArea>
                <CardContent>
                    <Typography variant="body2" color="textSecondary" component="p">
                        You are not logged in and changes cannot be saved.
                    </Typography>
                </CardContent>
            </CardActionArea>

            <CardActions>
                <Button 
                    className = {classes.button}
                    size = "small" 
                    onClick = { 
                        () => {
                            window.history.replaceState('#/', "", "#/login");
                            window.location.reload();
                        } 
                    }
                > Login </Button>
            </CardActions>
        </Card>
    );
}
