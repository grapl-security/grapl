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
    userName: Yup.string().required("Username Required"),
    password: Yup.string().required("Password Required"),
  });

  return (
    <div>
      <div>
        {" "}
        <GoogleSSO state={state} setState={setState} />{" "}
      </div>

      <div>
        <Formik
          initialValues={{ userName: "", password: "" }}
          validationSchema={validationSchema}
          onSubmit={async (values) => {
            const loginSuccess = await loginService(values.userName, values.password);

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
              <h3> User Login </h3>
              <Field name="userName" type="text" placeholder="Username" />
              {touched.userName && errors.userName && <div>{errors.userName}</div>}
              <Field name="password" type="password" placeholder="Password" /> <br />
              {touched.password && errors.password && <div>{errors.password}</div>}
              <button type="submit">SUBMIT</button>
              {state.loginFailed && <div>Unsuccessful Login</div>}
            </Form>
          )}
        </Formik>
      </div>
    </div>
  );
};

export default Login;
