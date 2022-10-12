import React from "react";
import LoginForm from "./LoginForm";

const Login = () => {
  return (
    <div>
      <h1 data-testid={"Grapl"}> Grapl </h1>
      <img src={"./assets/graplLogoFull.png"} alt={"Grapl Logo"} />
      <LoginForm />
    </div>
  );
};

export default Login;
