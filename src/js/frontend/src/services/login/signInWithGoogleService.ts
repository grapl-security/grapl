import DEV_API_EDGES from "../constants";
import { apiPostRequestWithBody } from "../fetch";

export const signInWithGoogleService = async (token: string) => {
  const body = JSON.stringify({
    token: token,
  });

  try {
    const loginData = await apiPostRequestWithBody(
      `${DEV_API_EDGES.auth}/sign_in_with_google`,
      body,
      "application/json"
    );
    return loginData["success"] === true;
  } catch (e) {
    console.log("Login Error", e);
    return false;
  }
};
