import { apiFetchWithBody } from "../fetch";
import DEV_API_EDGES from "../constants";

export const getNotebookUrl = async (): Promise<string> => {
    return apiFetchWithBody(`${DEV_API_EDGES.auth}/getNotebook`, "post")
        .then(
            (result) => result.success.notebook_url
        );
};
