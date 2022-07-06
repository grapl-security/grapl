import React from "react";

import { GoogleLogin } from "react-google-login";

import { LoginProps } from "types/CustomTypes";

import Icon from "@material-ui/core/Icon";
import Img from "../../assets/grapl_logo.svg";

import { loginStyles } from "./loginStyles";
import "./LogIn.css";


export const LogIn = (_: LoginProps) => {
    const useStyles = loginStyles;

    const classes = useStyles();

    const responseGoogle = (response: any) => {
        console.log(response);
    };

    return (
        <div className={classes.loginContainer}>

            <div className="grapl">
                <div>
                    <Icon>
                        <img className={classes.logoImage} src={Img} />
                    </Icon>
                </div>
            </div>

            <div>
                <div className="App">
                    <GoogleLogin
                        clientId="TBD"
                        buttonText="Login"
                        onSuccess={responseGoogle}
                        onFailure={responseGoogle}
                    />
                </div>
            </div>

        </div>
    );
};

export default LogIn;
