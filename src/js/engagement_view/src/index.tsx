import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import * as serviceWorker from "./serviceWorker";

import "./components/login/LogIn.css";
import { LogIn } from "./components/login/Login";
import { EngagementUx } from "./components/engagementView/EngagementView";
import Dashboard from "./components/dashboard/Dashboard";
import UploadPlugin from "./components/uploadPlugin/UploadPluginView";
import { HashRouter, Routes, Route } from "react-router-dom";


const rootElement = document.getElementById("root");

ReactDOM.render(
    <React.StrictMode>
            <HashRouter>
                <Routes>
                    <Route path="/login" element={LogIn} />
                    <Route path="/" element={Dashboard} />
                    <Route path="/plugins" element={UploadPlugin} />
                    <Route path="/engagements" element={EngagementUx} />
                </Routes>
            </HashRouter>
    </React.StrictMode>,
    rootElement
);

serviceWorker.unregister();
