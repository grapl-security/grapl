import React from "react";
import { GoogleLogin } from "@react-oauth/google";
import { signInWithGoogleService } from "../../services/login/signInWithGoogleService";

type Props = {
  state: { loginFailed: boolean };
  setState: React.Dispatch<
    React.SetStateAction<{
      loginFailed: boolean;
    }>
  >;
};

export const GoogleSSO = ({ state, setState }: Props) => {
  return (
    <div>
      <GoogleLogin
        data-testid="googleSSOButton"
        login_uri="/api/auth/signin_with_google"
        onSuccess={async (credentialResponse) => {
          if (credentialResponse.credential === undefined) {
            setState({
              ...state,
              loginFailed: true,
            });
            return;
          }

          const loginSuccess = await signInWithGoogleService(credentialResponse.credential);

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
        onError={() => {
          console.log("Login Failed");
        }}
      />
    </div>
  );
};
