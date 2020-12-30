import { PluginPayload } from "../../components/uploadPlugin/utils/uploadPluginTypes";
import {getModelPluginEdge} from "../getApiURLs"

import DEV_API_EDGES from '../constants';
import {apiFetch} from '../fetch';


export const getPluginList = async () => {
    // console.log("getModelPluginEdge", getModelPluginEdge())

 

    // const res = await fetch(`${getModelPluginEdge()}listModelPlugins`, 
    //     {
    //         method: 'POST',
    //         headers: {
    //             'Content-Type': 'application/json',
    //         },
    //         credentials: 'include',
    //     }
    // );
    
    // console.log("res", res);

    // const body = await res.json() as any;
    
    // console.log("body", body);

    try { 
        const getPluginList = await apiFetch(`${DEV_API_EDGES.modelPluginEdge}/listModelPlugins`, "POST");

        let pluginList: string[] = getPluginList.success.plugin_list;

        return pluginList;
    } catch (e) {
        console.warn("Error retrieving pluginList", e);
        return [];
    }
}
