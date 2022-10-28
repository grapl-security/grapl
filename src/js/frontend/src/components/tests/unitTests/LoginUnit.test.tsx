import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import "@testing-library/jest-dom/extend-expect";

import { screen, render } from "@testing-library/react";

import { act } from "react-dom/test-utils";
import LoginForm from "../../login/LoginForm";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
  test("Ensure username, password, submit button, and google SSO button render on screen", () => {
    const onSubmit = jest.fn();

    act(() => {
      render(
        <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
          <LoginForm onSubmit={onSubmit} />
        </GoogleOAuthProvider>,
      );
    });

    const username = screen.getByPlaceholderText(/Username/i);
    expect(username).toBeInTheDocument();

    const password = screen.getByPlaceholderText(/Password/i);
    expect(password).toBeInTheDocument();

    const submitButton = screen.getByRole("button", { name: /Submit/i });
    expect(submitButton).toBeInTheDocument();

    const googleSSOContainer = screen.getByTestId("googleSSOContainer");
    expect(googleSSOContainer).toBeInTheDocument();
  });
});
