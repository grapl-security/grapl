import React from "react";
import "../components/login/LogIn.css";
import { LogIn } from "../components/login/Login";
import { EngagementUx } from "../components/engagementView/EngagementView";
import Dashboard from "../components/dashboard/Dashboard";
import UploadPlugin from "../components/uploadPlugin/UploadPluginView";
import {  Routes, Route } from "react-router-dom";
import { HashRouter } from "react-router-dom";

export default function GraplRoutes() {
    console.log("Grapl loaded");

    return (
        <HashRouter>
            <Routes>
                <Route path="/login" element={<LogIn />} />
                <Route path="/" element={<Dashboard />} />
                <Route path="/plugins" element={<UploadPlugin />} />
                <Route path="/engagements" element={<EngagementUx />} />
            </Routes>
        </HashRouter>
    );
}