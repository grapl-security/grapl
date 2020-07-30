import React from 'react';
import './LogIn.css';
import {Field, Form, Formik} from "formik";
import {LoginProps} from '../src/modules/GraphViz/CustomTypes';
import {getAuthEdge} from './modules/GraphViz/engagement_edge/getApiURLs';
import * as Yup from "yup";
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles";

const useStyles = makeStyles(
  (theme: Theme) =>
      createStyles({
        valErrorMsg:{
          marginLeft: ".8rem",
          color: "red",
          fontSize: ".75rem"
        }
      }
  )
);

const engagement_edge = getAuthEdge();


const retry = async <T extends unknown>(f: () => T, count: number): Promise<T | undefined> => {
  if (count === 0) {
    return undefined
  }
  try {
    console.log('retrying')
    const x = await f();
    if (!x) {
      await new Promise(r => setTimeout(r, 1000));
      return await retry(f, count -1)
    } else {
      return x
    }
  } catch (e) {
    console.warn(e);
    await new Promise(r => setTimeout(r, 1000));
    return await retry(f, count -1)
  }

}


export const checkLogin = async (): Promise<boolean | null> => {

  try { 

    const res = await fetch(`${engagement_edge}checkLogin`, 
      {
        method: 'post',
        credentials: 'include',
      }
    );

    const body = await res.json();

    return (body['success'] === 'True')
  } catch (e) {
    console.warn(e);
    return null
  }
};

const validationSchema = Yup.object().shape({
  userName: Yup.string().required("Username Required"),
  password: Yup.string().required("Password Required")
})

export const LogIn = (_: LoginProps) => {
  const classes = useStyles()
  const [state, setState] = React.useState({
    loginFailed: false
  })
  return (
    <div className = "backgroundImage">
      <div className="grapl"> Grapl </div>
      <div className = "formContainer">
      <Formik
        initialValues={{ 
          userName: "", 
          password: "" 
        }}
        validationSchema = {validationSchema}
        onSubmit={ async values => {
          const password = await sha256WithPepper(
            values.userName, values.password
          );
          
          const loginSuccess = await login(values.userName, password);
          
          if (loginSuccess === true) {
            window.history.replaceState('#/login', "", "/")
            window.location.reload();
            console.log("Logged In")
          } else {
            setState({
              ...state,
              loginFailed: true
            })
          }
        }}
      >

      {({ errors, touched }) => (
        <Form>

            <Field name="userName" type="text" placeholder="Username"  />
            {touched.userName && errors.userName && <div className = {classes.valErrorMsg}>{errors.userName}</div>}
        
            <Field name="password" type="password" placeholder="Password"/> <br/>
            {touched.password && errors.password && <div className = {classes.valErrorMsg}>{errors.password}</div>}

            <button className="submitBtn"  type="submit">Submit</button>

            {state.loginFailed && <div className= {classes.valErrorMsg}>Unsuccessful Login</div>}
          </Form>
        )}
      </Formik>
        
      </div>
    </div>
  );
}

async function sha256(message: string) {
  // encode as UTF-8
  const msgBuffer = new TextEncoder().encode(message);

  // hash the message
  const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);

  // convert ArrayBuffer to Array
  const hashArray = Array.from(new Uint8Array(hashBuffer));

  // convert bytes to hex string
  return hashArray.map(b => ('00' + b.toString(16)).slice(-2)).join('');
}


const sha256WithPepper = async (username: string, password: string) => {
  // The pepper only exists to prevent rainbow tables for extremely weak passwords
  // Client side hashing itself is only to prevent cases where the password is
  // exposed before it makes it into the password database
  const pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254";
  let hashed = await sha256(password + pepper + username);

  for (let i = 0; i < 5000; i++) {
      hashed = await sha256(hashed)
  }
  return hashed
};

const login = async (username: string, password: string) => {
      try {
          const res = await fetch(`${engagement_edge}login`, {
              method: 'post',
              body: JSON.stringify({
                  'username': username,
                  'password': password
              }),
              headers: {
                  'Content-Type': 'application/json',
              },
              credentials: 'include',
          });
          
          const body = await res.json();

          return body['success'] === 'True';
        } catch (e) {
          console.log("Login Error", e);
          return false
      }
    };

export default LogIn;
