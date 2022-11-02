import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import "@testing-library/jest-dom";

import { screen, cleanup, fireEvent, render, waitFor, renderHook } from "@testing-library/react";

import { expect, test } from "@jest/globals";

import { act } from "react-dom/test-utils";
import LoginForm from "components/login/LoginForm";
import getTestPasswordFromAWSSecretsManager from "../../../services/test/modules/getSecretValues";
import { loginService } from "../../../services/login/loginService";
import { apiPostRequestWithBody } from "../../../services/fetch";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
  beforeEach(() => jest.clearAllMocks())
  afterEach(cleanup);

  test("Retrieves password from AWS and executes password in login form ", async () => {
    const onSubmit = jest.fn();


    act(() => {
      render(
        <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
          <LoginForm onSubmit={onSubmit} />
        </GoogleOAuthProvider>,
      );
    });

    const username = screen.getByPlaceholderText(/Username/i);
    const password = screen.getByPlaceholderText(/Password/i);
    const submitButton = screen.getByText("SUBMIT");

    const testUsername: string | undefined = process.env.GRAPL_TEST_USER_NAME;
    const testPassword: string | undefined = await getTestPasswordFromAWSSecretsManager;

    await waitFor(() => {
      fireEvent.change(username, {
        target: {
          value: testUsername,
        },
      });
    });

    await waitFor(() => {
      fireEvent.change(password, {
        target: {
          value: testPassword,
        },
      });
    });

    await waitFor(() => {
      fireEvent.click(submitButton);
    });

    await waitFor(() => {
      expect(username).not.toBeNull();
      expect(testPassword).not.toBeNull();
      expect(testPassword).toBeDefined();

      expect(onSubmit).toHaveBeenCalledWith({
        username: testUsername,
        password: testPassword,
      });
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
  });

  test("Test Login Call", async () => {
    const testUsername: string | undefined = process.env.GRAPL_TEST_USER_NAME;
    const testPassword: string | undefined = await getTestPasswordFromAWSSecretsManager;
    const webUiAddress = `${process.env["GRAPL_WEB_UI_ENDPOINT_ADDRESS"]}/`; // don't forget trailing `/`

    console.log("loggedIN", loggedIn);
  });


  screen.debug();
});

// actix-session
