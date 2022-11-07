// @ts-check
import { test, expect } from '@playwright/test';

test('Login Component has title and a form with username and password fields which are filled in with environment variables ' +
  'which check to see that we can submit our login form ', async ({ page }) => {
  await page.goto('localhost:1234/#/login');

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Grapl/);

  // create a locators
  const submitButton = page.getByRole('button', { name: 'Submit' });

  const testUsername = process.env.GRAPL_TEST_USER_NAME;
  const testPassword = process.env.GRAPL_TEST_PASSWORD;

  await page.getByPlaceholder('Username').fill(testUsername);
  await page.getByPlaceholder('Password').fill(testPassword);

  // Expect an attribute "to be strictly equal" to the value.

  // Click the Submit Button.
  await submitButton.click();

  // Expects the URL to contain intro.
  await expect(page).toHaveURL("http://localhost:1234/#/"); // redirects to homepage
});
