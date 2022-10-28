import React from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import LoginForm from "../components/login/LoginForm";

export default function GraplRoutes() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/"></Route>
        <Route path="/login" element={<LoginForm onSubmit={(formValue) => {}} />}></Route>
      </Routes>
    </HashRouter>
  );
}
