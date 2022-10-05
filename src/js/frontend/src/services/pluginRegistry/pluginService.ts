import DEV_API_EDGES from "../constants";
import { apiPostRequestWithBody } from "../fetch";

export const pluginService = async (pluginName: string, pluginType: string, eventSourceId: string) => {
  const createPlugin = JSON.stringify({
    plugin_name: pluginName,
    plugin_type: pluginType,
    event_source_id: eventSourceId,
  });

  try {
    const pluginData = await apiPostRequestWithBody(
      `${DEV_API_EDGES.plugin}/create`,
      createPlugin,
      "multipart/form-data",
    );
    return pluginData["success"] === true;
  } catch (e) {
    console.log("Login Error", e);
    return false;
  }
};
