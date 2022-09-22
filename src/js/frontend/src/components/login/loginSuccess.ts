import { CredentialResponse } from "@react-oauth/google";
import { signInWithGoogleService } from "../../services/login/signInWithGoogleService";

export const loginSuccess = async (
  state: { loginFailed: boolean },
  setState: React.Dispatch<React.SetStateAction<{ loginFailed: boolean }>>,
  credentialResponse: CredentialResponse,
): Promise<void> => {
  if (credentialResponse.credential === undefined) {
    setState({
      ...state,
      loginFailed: true,
    });
    return;
  }

  const loginSuccess = await signInWithGoogleService(credentialResponse.credential);

  if (loginSuccess) {
    window.history.replaceState("#/login", "", "#/");
    window.location.reload();
  } else {
    setState({
      ...state,
      loginFailed: true,
    });
  }
  return;
};
