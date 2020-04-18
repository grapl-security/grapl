import React from 'react';
import './LogIn.css';
import { LogIn } from './Login';
import { EngagementUx } from "./components/SideBar";
import { BrowserRouter, Switch,Route } from "react-router-dom";

export default function App() {
  return(
    <>
      <BrowserRouter>
        <Switch>
          <Route exact path="/" component={LogIn} />
          <Route path="/engagements" component={EngagementUx} />
        </Switch>
      </BrowserRouter>
    </>
  )
}

