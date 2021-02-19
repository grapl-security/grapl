import DEV_API_EDGES from '../constants';
import {apiFetch} from '../fetch';

export const getPluginList = async () => {
    try { 
        const getPluginListReq = await apiFetch(`${DEV_API_EDGES.modelPluginEdge}/listModelPlugins`, "POST");
        console.log("getPluginList", getPluginListReq)
        let pluginList: string[] = getPluginListReq.success.plugin_list;

        return pluginList;
    } catch (e) {
        console.warn("Error retrieving pluginList", e);
        return [];
    }
}
