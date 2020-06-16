import { RouteState, SetRouteState } from '../GraphViz/CustomTypes';

export const redirectTo = (
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

export const defaultRouteState = (): RouteState => {
    return {
        curPage: localStorage.getItem("grapl_curPage") || "login",
        lastCheckLoginCheck: Date.now()
    }
}
