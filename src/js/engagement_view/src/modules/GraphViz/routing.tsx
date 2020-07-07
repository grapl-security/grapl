import { RouteState } from '../GraphViz/CustomTypes';


export const defaultRouteState = (): RouteState => {
    return {
        curPage: localStorage.getItem("grapl_curPage") || "login",
        lastCheckLoginCheck: Date.now()
    }
}
