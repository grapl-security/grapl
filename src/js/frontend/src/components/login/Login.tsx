import React from "react";
import { GoogleLogin } from "@react-oauth/google";
import { signInWithGoogleService } from "../../services/login/signInWithGoogleService";

const Login = () => {
  const [state, setState] = React.useState({
    loginFailed: false,
  });

  return(
    <div>
      <div className="App">
        <GoogleLogin
          login_uri="/api/auth/signin_with_google"
          onSuccess={async (credentialResponse) => {
            if (credentialResponse.credential === undefined) {
              setState({
                ...state,
                loginFailed: true,
              });

              return;
            }

            const loginSuccess = await signInWithGoogleService(
              credentialResponse.credential
            );

            if (loginSuccess === true) {
              window.history.replaceState(
                "#/login",
                "",
                "#/"
              );
              window.location.reload();
              console.log("Logged In");
            } else {
              setState({
                ...state,
                loginFailed: true,
              });
            }
          }}
          onError={() => {
            console.log("Login Failed");
          }}
        />
      </div>
    </div>
  )
}

export default Login;