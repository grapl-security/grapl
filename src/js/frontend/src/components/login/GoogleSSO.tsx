import React from "react";
import { GoogleLogin } from "@react-oauth/google";
import { loginSuccess } from "./loginSuccess";

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
        onSuccess={(credentialResponse) => {
          loginSuccess(state, setState, credentialResponse).then((m) =>
            console.log("User Successfully logged in Using Google SSO", m)
          );
        }}
        onError={() => {
          console.log("Login Failed");
        }}
      />
    </div>
  );
};
