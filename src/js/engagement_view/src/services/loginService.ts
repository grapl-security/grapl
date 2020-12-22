import apiFetch from "./fetch";
import DEV_API_EDGES from "./constants";

export const checkLogin = async (): Promise<boolean | null> => {
  const loginData= await apiFetch(`${DEV_API_EDGES.auth}/checkLogin`, "post");
  return loginData['success'] === 'True';
};
