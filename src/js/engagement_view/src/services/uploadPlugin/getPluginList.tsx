import {apiFetch} from '../fetch';

export const getPluginList = async () => {
    try { 
        const getPluginListReq = await apiFetch(`modelPluginDeployer/listModelPlugins`, "POST");

        let pluginList: string[] = getPluginListReq.success.plugin_list;

        return pluginList;
    } catch (e) {
        console.warn("Error retrieving pluginList", e);
        return [];
    }
}
