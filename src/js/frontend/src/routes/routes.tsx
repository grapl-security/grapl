import React from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import Login from "../components/login/Login";

export default function GraplRoutes() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/"></Route>
        <Route path="/login" element={<Login />} />
      </Routes>
    </HashRouter>
  );
}
