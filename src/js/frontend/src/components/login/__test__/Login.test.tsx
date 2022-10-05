import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import { fireEvent, render, screen, waitFor, renderHook } from "@testing-library/react";

import Login from "components/login/Login";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login form", () => {
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

  test("Retrieves password and successfully logs in ", async () => {
    const { container } = render(
      <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
        <Login />
      </GoogleOAuthProvider>,
    );

    const username = screen.getByPlaceholderText(/Username/i);
    const password = screen.getByPlaceholderText(/Password/i);
    const submitButton = screen.getByRole("button", { name: /Submit/i });

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
          value:  process.env.GRAPL_TEST_USER_PASSWORD_SECRET_ID, // this is not the password secret value yet,
                                                                  // we have to interact with AWS SDK to get the value from the secret ID
        },
      });
    });

    await waitFor(() => {
      fireEvent.click(submitButton);
    });

    // const {result} = renderHook(() => useLoggedInUser())
    expect(process.env.GRAPL_TEST_USER_NAME).toBe('local-grapl-grapl-test-user');

    // expect(waitFor.innerHTML).toBe(
    //   '{"email":"mock@email.com","name":"mockname","color":"green"}'
    // )
  });
});
