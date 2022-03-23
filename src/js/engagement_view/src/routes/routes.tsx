import React from "react";
import { Routes, Route } from "react-router-dom";
import { HashRouter } from "react-router-dom";

import "../components/login/LogIn.css";

import { LogIn } from "../components/login/Login";
import { EngagementUx } from "../components/engagementView/EngagementView";

import Generators from "../components/generators/generators";
import Analyzers from "../components/analyzers/analyzers";

export default function GraplRoutes() {
    return (
        <HashRouter>
            <Routes>
                <Route path="/" element={<EngagementUx />} /> // Defaulting to
                Engagement UX for now, will replace with Dashboard eventually
                <Route path="/login" element={<LogIn />} />
                <Route path="/analyzers" element={<Analyzers />} />
                <Route path="/engagements" element={<EngagementUx />} />
                <Route path="/generators" element={<Generators />} />
            </Routes>
        </HashRouter>
    );
}
