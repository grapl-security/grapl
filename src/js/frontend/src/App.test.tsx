import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import { render, screen } from "@testing-library/react";

import Login from "components/login/Login";
import { GoogleSSO } from "components/login/GoogleSSO";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login form, submit button, and SSO Container appear in /#/login", () => {
  test("Look for Form Components", () => {
    render(
      <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
        <Login />
      </GoogleOAuthProvider>,
    );

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
