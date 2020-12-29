import { PluginPayload } from "./uploadPluginTypes";
import {getModelPluginEdge} from "../../../services/getApiURLs"

export const getPluginList = async () => {
    console.log("getModelPluginEdge", getModelPluginEdge())
    const res = await fetch(`${getModelPluginEdge()}listModelPlugins`, 
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        }
    );
    
    console.log("res", res);

    const body = await res.json() as any;
    
    console.log("body", body)
    try { 
        let pluginList: string[] = body.success.plugin_list;

        return pluginList
    } catch (e) {
        console.warn("Error retrieving pluginList", e);
        return [];
    }
}

export const deletePlugin = async ( pluginName: string ): Promise <boolean> => {
    const res = await fetch(`${getModelPluginEdge()}deleteModelPlugin`, 
        {
            method: 'POST',
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
    const res = await fetch(`${getModelPluginEdge()}deploy`, 
        {
            method: 'POST',
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