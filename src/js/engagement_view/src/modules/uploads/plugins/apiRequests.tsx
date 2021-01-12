import { PluginPayload } from "../plugins/uploadPluginTypes";
import {getModelPluginEdge} from "../../GraphViz/engagement_edge/getApiURLs"

export const getPluginList = async () => {
    const res = await fetch(`/prod/modelPluginDeployer/listModelPlugins`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        }
    );

    const body = await res.json() as any;

    try { 
        let pluginList: string[] = body.success.plugin_list;
        return pluginList
    } catch (e) {
        console.warn(e);
        return []
    }
}

export const deletePlugin = async ( pluginName: string ): Promise <boolean> => {
    const res = await fetch(`/prod/modelPluginDeployer/deleteModelPlugin`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
            body: JSON.stringify({plugins_to_delete: [pluginName]})
        }
    );
    await res.json();
    return true;
};

export const uploadFilesToDgraph = async (payload: PluginPayload ): Promise<boolean> => {
    const res = await fetch(`/prod/modelPluginDeployer/deploy`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
            body: JSON.stringify(payload)
        }
    );
    const body = await res.json();

    return body.success.Success;
};
