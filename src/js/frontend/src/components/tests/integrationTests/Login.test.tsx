import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import "@testing-library/jest-dom";

import { screen, cleanup, fireEvent, render, waitFor, renderHook } from "@testing-library/react";

import { expect, test } from "@jest/globals";

import { act } from "react-dom/test-utils";
import LoginForm from "components/login/LoginForm";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
  beforeEach(() => jest.clearAllMocks());
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

    const testUsername: string = "grapl-test-user";
    const testPassword: string = "grapl-test-password";

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

  screen.debug();
});
