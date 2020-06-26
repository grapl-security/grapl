import { RouteState, SetRouteState } from '../GraphViz/CustomTypes';

export const redirectTo = (
    routeState: RouteState, 
    setRouteState: SetRouteState, 
    pageName: string
    ) => {
    setRouteState({
        ...routeState,
        curPage: pageName,
    })
    console.log("Route State", routeState)
    // window.history.replaceState(routeState, "", pageName)

    // localStorage.setItem("grapl_curPage", page_name)
}

export const defaultRouteState = (): RouteState => {
    return {
        curPage: localStorage.getItem("grapl_curPage") || "login",
        lastCheckLoginCheck: Date.now()
    }
}
