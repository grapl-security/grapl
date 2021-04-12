import React from "react";
import "../components/login/LogIn.css";
import { LogIn } from "../components/login/Login";
import { EngagementUx } from "../components/engagementView/EngagementView";
import Dashboard from "../components/dashboard/Dashboard";
import UploadPlugin from "../components/uploadPlugin/UploadPluginView";
import { HashRouter, Route, Switch } from "react-router-dom";

export default function Routes() {
  console.log("Grapl loaded");

  return (
    <HashRouter>
      <Switch>
        <Route exact path="/login" component={LogIn} />
        <Route exact path="/" component={Dashboard} />
        <Route exact path="/plugins" component={UploadPlugin} />
        <Route exact path="/engagements" component={EngagementUx} />
      </Switch>
    </HashRouter>
  );
}
