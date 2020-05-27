import React, {useEffect} from 'react';
import './LogIn.css';
import {checkLogin, LogIn} from './Login';
import {EngagementUx} from "./components/SideBar";
import Dashboard from "./components/Dashboard";
import { RouteState, SetRouteState } from '../src/modules/GraphViz/CustomTypes';

console.log("App loading");

// Updates our react state, as well as localStorage state, to reflect the page
// we should render

const redirectTo = (
    routeState: RouteState, 
    setRouteState: SetRouteState, 
    page_name: string
    ) => {
    setRouteState({
        ...routeState,
        curPage: page_name,
    })
    localStorage.setItem("grapl_curPage", page_name)
}

const defaultRouteState = (): RouteState => {
    return {
        curPage: localStorage.getItem("grapl_curPage") || "login",
        lastCheckLoginCheck: Date.now(),
    }
}

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
            <LogIn loginSuccess={
                () => redirectTo(routeState, setRouteState, "dashboard")
            }></LogIn>
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

    

    if (routeState.curPage === "engagement_ux") {
        console.log("Routing to login page");
        return <EngagementUx/>
    }

    if(routeState.curPage === "dashboard"){
        console.log("Routing to Dashboard");
        return <Dashboard /> 
    }

    // #TODO: This should be a nice landing page explaining that something has gone
    // wrong, and give a redirect back to the login page
    console.warn("Invalid Page State");
    return <div>Invalid Page State</div>
    // <PageNotFound />
}


export default function App() {
    console.log("App loaded");
    return (
        <>
            <Router/>
        </>
    )
}

