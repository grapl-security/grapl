import { PluginPayload } from "../../types/uploadPluginTypes";
import DEV_API_EDGES from "../constants";
import {apiFetchPostRequest} from '../fetch';

export const uploadFilesToDgraph = async (payload: PluginPayload ): Promise<boolean> => {

    const dgraphPayload = JSON.stringify(payload);

    const dgraphFileUpload = await apiFetchPostRequest(`${DEV_API_EDGES.modelPluginEdge}/deploy`, "post", dgraphPayload);

    const pluginFiles = await dgraphFileUpload;

    return pluginFiles.success.Success;
};