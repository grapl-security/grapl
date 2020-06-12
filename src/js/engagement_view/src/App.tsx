import React, {useEffect} from 'react';
import './LogIn.css';
import {checkLogin, LogIn} from './Login';
import {EngagementUx} from "./components/SideBar";
import Dashboard from "./components/Dashboard";
import  PageNotFound  from "./components/PageNotFound";
import UploadPlugin from "./components/UploadPlugin";
import {redirectTo, defaultRouteState} from "../src/modules/GraphViz/routing";

// Updates our react state, as well as localStorage state, to reflect the page
// we should render

const Router = () => {
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
                            redirectTo(routeState, setRouteState, "login")
                        }
                    })
            }}
    });

    if (routeState.curPage === "login") {
        console.log("routing to Dashboard");
        return (
            <LogIn 
                loginSuccess={
                    () => redirectTo(routeState, setRouteState, "dashboard")
                }
            ></LogIn>
        )
    }

    // if (routeState.curPage === "dashboard") {
    //     console.log("routing to engagement_ux page");
    //     return (
    //         <LogIn loginSuccess={
    //             () => redirectTo(routeState, setRouteState, "engagement_ux")
    //         }></LogIn>
    //     )
    // }

    if (routeState.curPage === "engagementUX") {
        console.log("Routing to EngagementUX page");
        return <EngagementUx redirectTo={
            (pageName: string) => {
                redirectTo(routeState, setRouteState, pageName)
            }
        }/>
    }

    if(routeState.curPage === "dashboard"){
        console.log("Routing to Dashboard");
        return <Dashboard 
            redirectTo = {
                (pageName: string) => {
                    redirectTo(routeState, setRouteState, pageName)
                }
            }
        /> 
    }

    if(routeState.curPage === "uploadPlugin"){
        console.log("Routing to Upload Plugin");
        return <UploadPlugin 
            redirectTo = {
                (pageName: string) => {
                    redirectTo(routeState, setRouteState, pageName)
                }
            }
        /> 
    }
    console.warn("Invalid Page State");
    return <PageNotFound />
}


export default function App() {
    console.log("App loaded");
    return (
        <>
            <Router/>
        </>
    )
}

