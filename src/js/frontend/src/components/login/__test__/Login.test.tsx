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
          value: "local-grapl-grapl-test-user",
        },
      });
    });

    await waitFor(() => {
      fireEvent.change(password, {
        target: {
          value: "local-grapl-password", // this needs to come from the environment from secrets manager
        },
      });
    });

    await waitFor(() => {
      fireEvent.click(submitButton);
    });

    // const {result} = renderHook(() => useLoggedInUser())
    // expect(result.current).toEqual({name: 'Alice'})

    // expect(waitFor.innerHTML).toBe(
    //   '{"email":"mock@email.com","name":"mockname","color":"green"}'
    // )
  });
});