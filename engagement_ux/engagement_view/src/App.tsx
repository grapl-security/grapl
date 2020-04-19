import React from 'react';
import './LogIn.css';
import { LogIn } from './Login';
import { EngagementUx } from "./components/SideBar";

// Updates our react state, as well as localStorage state, to reflect the page
// we should render
const redirectTo = (routeState: any, setRouteState: any, page_name: string) => {
  setRouteState({
    curPage: page_name,
  })
  localStorage.setItem("grapl_curPage", page_name)
}

const Router = ({}: any) => {
  // By default, load either the last page we were on, or the login page
  // if there is no last page
  const [routeState, setRouteState] = React.useState({
    curPage: localStorage.getItem("grapl_curPage") || "login",
  })

  if(routeState.curPage === "login") {
    return <LogIn loginSuccess={
      () => redirectTo(routeState, setRouteState, "engagement_ux")
    }></LogIn>
  }

  if (routeState.curPage === "engagement_ux") {
      return <EngagementUx/>
  }

  // #TODO: This should be a nice landing page explaining that something has gone
  // wrong, and give a redirect back to the login page
  return <div>"Invalid Page State"</div>
}


export default function App() {


  return(
    <>
      <Router></Router>
    </>
  )
}

