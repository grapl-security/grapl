import { apiFetch } from "/home/colin/grapl-ws/grapl/src/js/engagement_view/src/services/fetch"

export const getNotebookUrl = async (): Promise<string> => {
  return apiFetch(`/prod/auth/getNotebook`, "post").then(
    (result: any) => result.success.notebook_url
  );
};
