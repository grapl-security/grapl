import DEV_API_EDGES from "../constants";
import { apiPostRequestStreamWithBody } from "../fetch";

export const createPluginService = async (filename: string, analyzername: any) => {
    try {
        const createPluginReq = await apiPostRequestStreamWithBody(
            `${DEV_API_EDGES.plugin}/create`,
            "POST"
        );
        console.log("getPluginList", createPluginReq);

        let success: string[] = createPluginReq.success.Success;

        return success;
    } catch (e) {
        console.warn("Error creating plugin", e);
        return [];
    }
};
