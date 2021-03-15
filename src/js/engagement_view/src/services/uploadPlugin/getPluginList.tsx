import DEV_API_EDGES from "../constants";
import { apiFetchWithBody } from "../fetch";

export const getPluginList = async () => {
	try {
		const getPluginListReq = await apiFetchWithBody(
			`${DEV_API_EDGES.modelPluginEdge}/listModelPlugins`,
			"POST"
		);
		console.log("getPluginList", getPluginListReq);
		let pluginList: string[] = getPluginListReq.success.plugin_list;

		return pluginList;
	} catch (e) {   
		console.warn("Error retrieving pluginList", e);
		return [];
	}
};
