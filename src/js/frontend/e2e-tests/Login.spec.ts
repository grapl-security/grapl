// @ts-check
import { test, expect } from "@playwright/test";
import getValueFromAWSSecretsManager from "../../frontend/src/services/test/modules/getSecretValues";


test.describe("navigation", () => {
  // get web UI address from environment variable in Nomad job
  const webUIAddress = process.env.GRAPL_WEB_UI_ENDPOINT_ADDRESS;

  test.beforeEach(async ({ page }) => {
    // Given a user who navigates to the login page
    await page.goto(`${webUIAddress}/#/login`);
  });

  test("Login Component has title and a form with username and password fields which are filled in with environment variables " +
    "which check to see that we can submit our login form ", async ({ page }) => {
    await expect(page).toHaveTitle(/Grapl/);

    const testUsername = process.env.GRAPL_TEST_USER_NAME;
    const testPasswordSecretId = process.env.GRAPL_TEST_USER_PASSWORD_SECRET_ID;
    const testPassword = await getValueFromAWSSecretsManager(testPasswordSecretId);
    const submitButton = page.getByRole("button", { name: "Submit" });

    // When a user fills their username and password into the "Username" and "Password" form values and clicks the submit button
    await page.getByPlaceholder("Username").fill(testUsername);
    await page.getByPlaceholder("Password").fill(testPassword);
    await submitButton.click();

    // Then our application will redirect to homepage indicating successful login.
    await expect(page).toHaveURL(`${webUIAddress}/#/`);
  });

  test("ensure user recieves error when they submit a form with incorrect credentials", async ({ page }) => {
    const submitButton = page.getByRole("button", { name: "Submit" });
    // Given a user who navigates to the login component in our app
    await page.goto(`${webUIAddress}/#/login`);
    // When the user submits the form with an incorrect username and password
    await page.getByPlaceholder("Username").fill("fakeUsername");
    await page.getByPlaceholder("Password").fill("fakePassword");
    await submitButton.click();
    // Then the user will see a message telling them their login was unsuccessful
    await expect(page.getByTestId("loginError")).toContainText("Login Unsuccessful");
  });
});





