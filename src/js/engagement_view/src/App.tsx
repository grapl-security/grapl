import React from 'react';
import './LogIn.css';
import { LogIn } from './Login';
import { EngagementUx } from "./components/EngagementView";
import Dashboard from "./components/Dashboard";
import UploadPlugin from "./components/UploadPlugin";
import { HashRouter, Route, Switch } from 'react-router-dom';

// Updates our react state, as well as localStorage state, to reflect the page
// we should render

export default function App() {
    console.log("Grapl loaded");

    return (
        <>
            <HashRouter>
                <Switch>
                    <Route exact path = "/login" component = {LogIn}/>
                    <Route exact path = "/" component = {Dashboard}/>
                    <Route exact path = "/plugins" component = {UploadPlugin}/>
                    <Route exact path = "/engagements" component = {EngagementUx}/>
                </Switch>
            </HashRouter>
        </>
    )
}

