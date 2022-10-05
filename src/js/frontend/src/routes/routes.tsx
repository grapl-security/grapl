import React from "react";
import { Routes, Route } from "react-router-dom";
import { HashRouter } from "react-router-dom";
import Login from "../components/login/Login";
import Plugins from "../components/plugins/Plugins";

export default function GraplRoutes() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/"></Route>
        <Route path="/login" element={<Login />} />
        <Route path="/plugins" element={<Plugins />} />
      </Routes>
    </HashRouter>
  );
}
