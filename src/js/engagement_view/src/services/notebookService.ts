import {apiFetch} from "./fetch";

export const getNotebookUrl = async (): Promise<string> => {
  return apiFetch(`/prod/auth/getNotebook`, "post").then(
    (result: any) => result.success.notebook_url
  );
};
