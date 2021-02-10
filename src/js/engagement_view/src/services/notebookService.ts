import apiFetch from "./fetch";

export const getNotebookUrl = async (): Promise<string> => {
  return apiFetch(`/prod/auth/getNotebook`, "post").then(
    (result) => result.success.notebook_url
  );
};
