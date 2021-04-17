import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import { createMuiTheme, ThemeProvider } from "@material-ui/core/styles";
import App from "./App";
import * as serviceWorker from "./serviceWorker";
import { HashRouter } from "react-router-dom";

const darkTheme = createMuiTheme({
    palette: {
        type: "dark",
        primary: {
            main: "#373740",
        },
    },
});

const rootElement = document.getElementById("root");

ReactDOM.render(
    <React.StrictMode>
        <HashRouter>
            <ThemeProvider theme={darkTheme}>
                <App />,
            </ThemeProvider>
        </HashRouter>
    </React.StrictMode>,
    rootElement
);

serviceWorker.unregister();
