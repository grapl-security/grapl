import React from "react";
import { GoogleOAuthProvider } from "@react-oauth/google";

import {
  screen,
  cleanup,
  fireEvent,
  render,
} from "@testing-library/react";

import { act } from "react-dom/test-utils";
import Login from "components/login/Login";

// to show the result of render(), use screen.debug() which displays HTML
describe("Login Component", () => {
  afterEach(cleanup);

  test("Retrieves password and successfully logs in ", async () => {

    act(() => {
      render(
        <GoogleOAuthProvider clientId="340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com">
          <Login />
        </GoogleOAuthProvider>,
      );
    });


    const username = screen.getByPlaceholderText(/Username/i);
    const password = screen.getByPlaceholderText(/Password/i);
    const submitButton = screen.getByText("SUBMIT");

    act(() => {
      fireEvent.change(username, {
        target: {
          value: process.env.GRAPL_TEST_USER_NAME,
        },
      });
    });

    act(() => {
      fireEvent.change(password, {
        target: {
          value: process.env.GRAPL_TEST_USER_PASSWORD_SECRET_ID, // this is not the password secret value yet,// we have to interact with AWS SDK to get the value from the secret ID
        },
      });
    });

    act(() => {
      fireEvent.click(submitButton);
    });

  console.log("process.env.GRAPL_TEST_USER_NAME", process.env.GRAPL_TEST_USER_NAME)
    expect(process.env.GRAPL_TEST_USER_NAME).toBe("local-grapl-grapl-test-user");
    expect(process.env.GRAPL_TEST_PASSWORD).not.toBeNull()


    screen.debug();
  });


});


