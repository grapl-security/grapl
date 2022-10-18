import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import "@testing-library/jest-dom";

import { screen, cleanup, fireEvent, render, waitFor } from "@testing-library/react";

import { expect, test } from "@jest/globals";

import { act } from "react-dom/test-utils";
import LoginForm from "components/login/Login";

import getAwsClient from "../../../services/test/modules/envHelpers";
import { SecretsManagerClient } from "@aws-sdk/client-secrets-manager";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
  afterEach(cleanup);

  test("Retrieves password from AWS and executes password in login form ", async () => {
    act(() => {
      render(
        <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
          <LoginForm />
        </GoogleOAuthProvider>,
      );
    });

    const username = screen.getByPlaceholderText(/Username/i);
    const password = screen.getByPlaceholderText(/Password/i);
    const submitButton = screen.getByText("SUBMIT");

    let getCreds = getAwsClient(SecretsManagerClient);

    await waitFor(() => {
      fireEvent.change(username, {
        target: {
          value: process.env.GRAPL_TEST_USER_NAME,
        },
      });
    });

    // make call to AWS secrets manager
    // get secret stored upder this id: GRAPL_TEST_USER_PASSWORD_SECRET_ID

    await waitFor(() => {
      fireEvent.change(password, {
        target: {
          value: getCreds.GRAPL_TEST_USER_PASSWORD_SECRET_ID,
        },
      });
    });

    await waitFor(() => {
      fireEvent.click(submitButton);
    });

    await waitFor(() => {
      expect(username).not.toBeNull();
      expect(getCreds).not.toBeNull();
      expect(getCreds).toBeDefined();
    });

    screen.debug();
  });
});
