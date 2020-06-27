import React, {useEffect} from 'react';
import './LogIn.css';
import {checkLogin, LogIn} from './Login';
import {EngagementUx} from "./components/SideBar";
import Dashboard from "./components/Dashboard";
import PageNotFound  from "./components/PageNotFound";
import UploadPlugin from "./components/UploadPlugin";
import {defaultRouteState} from "../src/modules/GraphViz/routing";
import {HashRouter, Route, Switch, Link} from 'react-router-dom';

// Updates our react state, as well as localStorage state, to reflect the page
// we should render

const RouterComponent = () => {
    // By default, load either the last page we were on, or the login page
    // if there is no last page
    const [routeState, setRouteState] = React.useState(defaultRouteState())

    useEffect(() => {
        if (routeState.curPage !== "login") {
            if (Date.now() - routeState.lastCheckLoginCheck > 1000) {
                checkLogin()
                    .then((loggedIn) => {
                        console.log('Not logged in, redirecting.');
                        if (!loggedIn && routeState.curPage !== "login") {
                            window.history.replaceState('/login', "", "/dashboard")
                        }
                    })
            }}
    });


    console.warn("Invalid Page State");
    // return <PageNotFound />
}
export default function App() {
    console.log("App loaded");
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

