import { PluginPayload } from "../../types/uploadPluginTypes";
import {apiFetchWithBody} from '../fetch';

export const uploadFilesToDgraph = async (payload: PluginPayload ): Promise<boolean> => {

    const dgraphPayload = JSON.stringify(payload);

    const dgraphFileUpload = await apiFetchWithBody(`modelPluginDeployer/deploy`, "post", dgraphPayload);

    const pluginFiles = await dgraphFileUpload;

    return pluginFiles.success.Success;
};
