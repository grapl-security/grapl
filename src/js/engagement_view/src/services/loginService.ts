import apiFetch from "./fetch";
import DEV_API_EDGES from "./constants";

export const checkLogin = (): Promise<boolean | null> => {
  return apiFetch(`${DEV_API_EDGES.auth}/checkLogin`, "post");
};
