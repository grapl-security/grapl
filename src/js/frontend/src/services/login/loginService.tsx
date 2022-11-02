import DEV_API_EDGES from "../constants";
import { apiPostRequestWithBody } from "../fetch";

export const loginService = async (username: string | undefined, password: string | undefined) => {
  const loginBody = JSON.stringify({
    username: username,
    password: password,
  });

  try {
    const loginData = await apiPostRequestWithBody(
      `${DEV_API_EDGES.auth}/sign_in_with_password`,
      loginBody,
    );
    console.log("Login Data", loginData);
    return loginData;
  } catch (e) {
    console.log("Login Error", e);
    return e;
  }
};
