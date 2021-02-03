import React from "react";
import Routes from "./routes";

// Updates our react state, as well as localStorage state, to reflect the page
// we should render

export default function App() {
  console.log("Grapl loaded");

  return (
    <>
      <Routes />
    </>
  );
}
