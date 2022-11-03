import React from "react";
import { Field, Form, Formik } from "formik";
import { GoogleSSO } from "./GoogleSSO";

import { loginService } from "../../services/login/loginService";
import { yupValidationSchema } from "./yupValidationSchema";

export interface FormValues {
  username: string;
  password: string;
}

export interface FormProps {
  onSubmit: (formValue: FormValues) => void;
}

const LoginForm = ({ onSubmit }: FormProps) => {
  const [state, setState] = React.useState({
    userLoggedIn: false, // Boolean represented as true when user is successfully authenticated,
                        // false when there's an auth error, token has been removed or user has been logged out
  });

  const handleSubmit = async (values: FormValues) => {
    const loginServiceResponse = await loginService(values.username, values.password);

    if (loginServiceResponse["success"]) {
      window.history.replaceState("#/login", "", "#/");
      window.location.reload();
    } else {
      setState({
        ...state,
        userLoggedIn: true,
      });
    }
    onSubmit(values); // for jest mocks
  };

  return (
    <div data-testid="googleSSOContainer">
      <h1 data-testid={"Grapl"}> Grapl </h1>

      <img src={"./assets/graplLogoFull.svg"} alt={"Grapl Logo"} />

      <GoogleSSO state={state} setState={setState} />

      <Formik
        data-testid="form"
        initialValues={{ username: "", password: "" }}
        validationSchema={yupValidationSchema}
        onSubmit={handleSubmit}
      >
        {({ errors, touched }) => (
          <Form>
            <Field data-testid={"username"} name="username" type="text" placeholder="Username" />
            {touched.username && errors.username && <div>{errors.username}</div>}
            <Field
              data-testid={"password"}
              name="password"
              s
              type="password"
              placeholder="Password"
            />{" "}
            <br />
            {touched.password && errors.password && <div>{errors.password}</div>}
            <button data-testid={"button"} name="submitButton" type="submit">
              SUBMIT
            </button>
            {state.userLoggedIn && <div data-testid={"loginError"}>Unsuccessful Login</div>}
          </Form>
        )}
      </Formik>
    </div>
  );
};

export default LoginForm;
