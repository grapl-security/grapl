import {apiFetchWithBody} from '../fetch';

export const deletePlugin = async ( pluginName: string ): Promise <boolean> => {
    const body = JSON.stringify( {plugins_to_delete: [pluginName]} );
    
    try{ 
        const response = await apiFetchWithBody(`modelPluginDeployer/deleteModelPlugin`, "post", body);
    
        await response.success;
        return true; 
    } catch (e){
        console.warn("Error deleting plugin", e);
        return false; 
    }
};
