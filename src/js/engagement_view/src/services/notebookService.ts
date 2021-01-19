import apiFetch from "./fetch";
import DEV_API_EDGES from "./constants";

export const getNotebookUrl = async (): Promise<string> => {
  return apiFetch(`/prod/auth/getNotebook`, "post").then(
    (result) => result.success.notebook_url
  );
};
