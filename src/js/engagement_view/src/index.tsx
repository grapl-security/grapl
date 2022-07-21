import React from "react";
import ReactDOM from "react-dom";

import "./index.css";

import { createTheme, ThemeProvider } from "@material-ui/core/styles";

import { GoogleOAuthProvider } from "@react-oauth/google";

import App from "./App";
import theme from "./theme";
import * as serviceWorker from "./serviceWorker";

const rootElement = document.getElementById("root");

// TODO(inickles): parameterize and templetize the clientId with the backend
ReactDOM.render(
    <React.StrictMode>
        <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
            <ThemeProvider theme={theme}>
                <App />
            </ThemeProvider>
        </GoogleOAuthProvider>
    </React.StrictMode>,
    rootElement
);

serviceWorker.unregister();
