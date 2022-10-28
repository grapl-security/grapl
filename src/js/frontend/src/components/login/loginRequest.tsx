import { CredentialResponse } from "@react-oauth/google";
import { signInWithGoogleService } from "../../services/login/signInWithGoogleService";

export const loginRequest = async (
  state: { userLoggedIn: boolean },
  setState: React.Dispatch<React.SetStateAction<{ userLoggedIn: boolean }>>,
  credentialResponse: CredentialResponse,
): Promise<void> => {
  if (credentialResponse.credential === undefined) {
    setState({
      ...state,
      userLoggedIn: false,
    });
    return;
  }

  const successfulLoginResponse = await signInWithGoogleService(credentialResponse.credential);

  if (successfulLoginResponse) {
    window.history.replaceState("#/login", "", "#/");
    window.location.reload();
  } else {
    setState({
      ...state,
      userLoggedIn: true,
    });
  }
  return;
};
