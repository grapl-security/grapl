import React from 'react';
import * as Yup from "yup";
import {Field, Form, Formik} from "formik";

import './LogIn.css';
import {loginStyles} from './styles';
import {LoginProps} from '../../types/CustomTypes';
import {engagementEdgeLoginReq} from '../../apiRequests/engagementEdgeLoginReq';
import {sha256WithPepper} from './utils/passwordHashing';


export const LogIn = (_: LoginProps) => {
  const useStyles = loginStyles; 
  
  const classes = useStyles();

  const validationSchema = Yup.object().shape({
    userName: Yup.string().required("Username Required"),
    password: Yup.string().required("Password Required")
  })
  
  const [state, setState] = React.useState({
    loginFailed: false
  })

  return (
    <div className = "backgroundImage">
      <div className = "grapl"> Grapl </div>

      <div className = "formContainer">
        <Formik
          initialValues={ {userName: "", password: ""} }
          validationSchema = {validationSchema}
          onSubmit = { 
            async values => {
              const password = await sha256WithPepper(
                values.userName, values.password
              );
              
              const loginSuccess = await engagementEdgeLoginReq(values.userName, password);
              
              if (loginSuccess === true) {
                window.history.replaceState('#/login', "", "#/")
                window.location.reload();
                console.log("Logged In");
              } else {
                setState({
                  ...state,
                  loginFailed: true
                })
              }
            }
          }
        >

          {
            ({ errors, touched }) => (
                <Form>
                  <Field name="userName" type="text" placeholder="Username"  />
                  {touched.userName && errors.userName && <div className = {classes.valErrorMsg}>{errors.userName}</div>}
              
                  <Field name="password" type="password" placeholder="Password"/> <br/>
                  {touched.password && errors.password && <div className = {classes.valErrorMsg}>{errors.password}</div>}

                  <button className="submitBtn"  type="submit">Submit</button>

                  {state.loginFailed && <div className= {classes.valErrorMsg}>Unsuccessful Login</div>}
                </Form>
            )
          }
        </Formik>
      </div>
    </div>
  );
}

export default LogIn;