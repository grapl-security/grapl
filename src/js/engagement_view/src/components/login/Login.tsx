import React from "react";
import * as Yup from "yup";
import { Field, Form, Formik } from "formik";
import { loginService } from "services/login/loginService";

import { LoginProps } from "types/CustomTypes";

import Icon from "@material-ui/core/Icon";
import Img from "../../assets/grapl_logo.svg";

import { loginStyles } from "./loginStyles";
import "./LogIn.css";

export const LogIn = (_: LoginProps) => {
    const useStyles = loginStyles;

    const classes = useStyles();

    const validationSchema = Yup.object().shape({
        userName: Yup.string().required("Username Required"),
        password: Yup.string().required("Password Required"),
    });

    const [state, setState] = React.useState({
        loginFailed: false,
    });

    return (
        <div className={classes.loginContainer}>
            <div className="grapl">
                <div>
                    <Icon>
                        <img className={classes.logoImage} src={Img} />
                    </Icon>
                </div>
            </div>

            <div className={classes.formContainer}>
                <Formik
                    initialValues={{ userName: "", password: "" }}
                    validationSchema={validationSchema}
                    onSubmit={async (values) => {
                        const loginSuccess = await loginService(
                            values.userName,
                            values.password
                        );

                        if (loginSuccess === true) {
                            window.history.replaceState("#/login", "", "#/");
                            window.location.reload();
                            console.log("Logged In");
                        } else {
                            setState({
                                ...state,
                                loginFailed: true,
                            });
                        }
                    }}
                >
                    {({ errors, touched }) => (
                        <Form>
                            <h3 className={classes.loginText}> User Login </h3>
                            <Field
                                className={classes.field}
                                name="userName"
                                type="text"
                                placeholder="Username"
                            />
                            {touched.userName && errors.userName && (
                                <div className={classes.valErrorMsg}>
                                    {errors.userName}
                                </div>
                            )}
                            <Field
                                name="password"
                                type="password"
                                placeholder="Password"
                            />{" "}
                            <br />
                            {touched.password && errors.password && (
                                <div className={classes.valErrorMsg}>
                                    {errors.password}
                                </div>
                            )}
                            <button className={classes.submitBtn} type="submit">
                                SUBMIT
                            </button>
                            {state.loginFailed && (
                                <div className={classes.valErrorMsg}>
                                    Unsuccessful Login
                                </div>
                            )}
                        </Form>
                    )}
                </Formik>
            </div>
        </div>
    );
};

export default LogIn;
