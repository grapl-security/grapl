import React from "react";
import ReactDOM from "react-dom";

import "./index.css";

import { ThemeProvider } from "@material-ui/core/styles";

import App from "./App";
import theme from "./theme";
import * as serviceWorker from "./serviceWorker";

const rootElement = document.getElementById("root");

ReactDOM.render(
    <React.StrictMode>
        <ThemeProvider theme={theme}>
            <App />
        </ThemeProvider>
    </React.StrictMode>,
    rootElement
);

serviceWorker.unregister();
