import React from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import LoginForm from "../components/login/LoginForm";
import Dashboard from "../components/dashboard";

export default function GraplRoutes() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Dashboard />}></Route>
        <Route path="/login" element={<LoginForm onSubmit={(formValue) => {}} />}></Route>
      </Routes>
    </HashRouter>
  );
}
