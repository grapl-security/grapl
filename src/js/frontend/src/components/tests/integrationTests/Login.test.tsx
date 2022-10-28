import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import "@testing-library/jest-dom";

import { screen, cleanup, fireEvent, render, waitFor } from "@testing-library/react";

import { expect, test } from "@jest/globals";

import { act } from "react-dom/test-utils";
import LoginForm from "components/login/LoginForm";
import getTestPasswordFromAWSSecretsManager from "../../../services/test/modules/getSecretValues";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
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

    const testPassword = await getTestPasswordFromAWSSecretsManager;

    await waitFor(() => {
      fireEvent.change(username, {
        target: {
          value: process.env.GRAPL_TEST_USER_NAME,
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
        username: process.env.GRAPL_TEST_USER_NAME,
        password: testPassword,
      });
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
  });
});
