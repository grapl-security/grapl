import React from 'react';
import './LogIn.css';
import {Field, Form, Formik} from "formik";
import {LoginProps} from '../src/modules/GraphViz/CustomTypes';
import {getAuthEdge} from './modules/GraphViz/engagement_edge/getApiURLs';

const engagement_edge = getAuthEdge();

export const checkLogin = async () => {
    const res = await fetch(`${engagement_edge}checkLogin`, {
        method: 'get',
        credentials: 'include',
    });

    const body = await res.json();

    return body['success'] === 'True';
};

export const LogIn = (_: LoginProps) => {
  return (
    <div className = "backgroundImage">
      <div className="grapl"> Grapl </div>
      <div className = "formContainer">
      <Formik
        initialValues={{ userName: "", password: "" }}
        onSubmit={async values => {
          console.log("logging in");
          const password = await sha256WithPepper(
            values.userName, values.password
          );
          
          const loginSuccess = login(values.userName, password);
          
          if (await loginSuccess) {
            window.history.replaceState('/login', "", "/")
            window.location.reload();
            console.log("Logged in");
            console.log("LoginSuccess", loginSuccess)
          } else {
            console.warn("Login failed!")
          }
        }}
      >
        <Form>
          <Field name="userName" type="text" placeholder="Username" /> <br/>
          <Field name="password" type="password" placeholder="Password"/> <br/>
          <button className="submitBtn"  type="submit"> Submit </button>
        </Form>
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
          console.log("body", body);
          return body['success'] === 'True';
        } catch (e) {
          console.log(e);
          return false
      }
    };

export default LogIn;
