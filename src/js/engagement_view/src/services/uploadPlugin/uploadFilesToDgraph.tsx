import { PluginPayload } from "../../components/uploadPlugin/utils/uploadPluginTypes";
import DEV_API_EDGES from "../constants";
import {apiPostFetch} from '../fetch';

export const uploadFilesToDgraph = async (payload: PluginPayload ): Promise<boolean> => {

    const dgraphPayload = JSON.stringify(payload);

    const dgraphFileUpload = await apiPostFetch(`${DEV_API_EDGES.modelPluginEdge}/deploy`, "post", dgraphPayload);

    const pluginFiles = await dgraphFileUpload.json();

    return pluginFiles.success.Success;
};