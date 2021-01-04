import DEV_API_EDGES from '../constants';
import {apiFetchPostRequest} from '../fetch';

export const deletePlugin = async ( pluginName: string ): Promise <boolean> => {
    const body = JSON.stringify( {plugins_to_delete: [pluginName]} );
    
    try{ 
        const response = await apiFetchPostRequest(`${DEV_API_EDGES.modelPluginEdge}/deleteModelPlugin`, "post", body);
    
        await response.success;
        return true; 
    } catch (e){
        console.warn("Error deleting plugin", e);
        return false; 
    }
};
