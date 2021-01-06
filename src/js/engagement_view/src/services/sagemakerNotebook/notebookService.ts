import  { apiFetch } from "../fetch";
import DEV_API_EDGES from "../constants";

export const getNotebookUrl = async (): Promise<string> => {
  return apiFetch(`${DEV_API_EDGES.auth}/getNotebook`, "post")
  .then(
    (result) => result.success.notebook_url
  );
};
