import {apiFetch} from "./fetch";

export const checkLogin = async (): Promise<boolean | null> => {
  try {
    const loginData = await apiFetch(`/prod/auth/checkLogin`, "post");
    return loginData['success'] === 'True';
  } catch (e) {
    return null;
  }
};
