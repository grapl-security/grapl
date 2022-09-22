import React from "react";
import * as Yup from "yup";
import { Field, Form, Formik } from "formik";
import { GoogleSSO } from "./GoogleSSO";

import { loginService } from "../../services/login/loginService";

const Login = () => {
  const [state, setState] = React.useState({
    loginFailed: false,
  });

  const validationSchema = Yup.object().shape({
    username: Yup.string().required("Username Required"),
    password: Yup.string().required("Password Required"),
  });


  return (
    <div data-testid="googleSSOContainer">
        <GoogleSSO state={state} setState={setState}/>

      <div>
        <Formik
          initialValues={{ username: "", password: "" }}
          validationSchema={validationSchema}
          onSubmit={async (values) => {
            const loginSuccess = await loginService(values.username, values.password);

            if (loginSuccess) {
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
              <h1> Grapl </h1>
              <Field name="username" type="text" placeholder="Username" />
              {touched.username && errors.username && <div>{errors.username}</div>}
              <Field name="password" type="password" placeholder="Password" /> <br />
              {touched.password && errors.password && <div>{errors.password}</div>}
              <button name="submitButton" type="submit">SUBMIT</button>
              {state.loginFailed && <div>Unsuccessful Login</div>}
            </Form>
          )}
        </Formik>
      </div>
    </div>
  );
};

export default Login;
