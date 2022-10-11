import React from "react";
import { Field, Form, Formik } from "formik";
import { GoogleSSO } from "./GoogleSSO";

import { loginService } from "../../services/login/loginService";
import { yupValidationSchema } from "./yupValidationSchema";

const Login = () => {
  const [state, setState] = React.useState({
    loginFailed: false, // Boolean represented as true when user is successfully authenticated,
    // false when there's an auth error, token has been removed or user has been logged out
  });

  return (
    <div data-testid="googleSSOContainer">
      <GoogleSSO state={state} setState={setState} />

      <div>
        <Formik

          initialValues={{ username: "", password: "" }}
          validationSchema={yupValidationSchema}
          onSubmit={
            async (values) => {
              const loginSuccess = await loginService(values.username, values.password);

              if (loginSuccess) {
                {!state.loginFailed && <div data-testid={"loginSuccess"}> Logging in</div>}
                window.history.replaceState("#/login", "", "#/");
                window.location.reload();
                console.log("User successfully logged in using Login Form.");
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
              <h1 data-testid={"Grapl"}> Grapl </h1>
              <Field data-testid={"username"} name="username" type="text" placeholder="Username" />
              {touched.username && errors.username && <div>{errors.username}</div>}

              <Field data-testid={"password"} name="password" type="password" placeholder="Password" /> <br />
              {touched.password && errors.password && <div>{errors.password}</div>}

              <button data-testid={"button"} name="submitButton" type="submit">
                SUBMIT
              </button>
              {state.loginFailed && <div data-testid={"loginError"}>Unsuccessful Login</div>}
            </Form>
          )}
        </Formik>
      </div>
    </div>
  );
};

export default Login;
