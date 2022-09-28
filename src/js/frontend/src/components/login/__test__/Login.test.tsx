import { GoogleOAuthProvider } from "@react-oauth/google";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from '@testing-library/user-event';


import Login from "components/login/Login";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login", () => {
  it("submits correct values", async () => {
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
          value: "local-grapl-grapl-test-user"
        }
      })
    })

    await waitFor(() => {
      fireEvent.change(password, {
        target: {
          value: "mock@email.com"
        }
      })
    })


    await waitFor(() => {
      fireEvent.click(submitButton)
    })

    expect(waitFor.innerHTML).toBe(
    )
  })
});