// ***********************************************************
// This example support/index.js is processed and
// loaded automatically before your test files.
//
// This is a great place to put global configuration and
// behavior that modifies Cypress.
//
// You can change the location of this file or turn off
// automatically serving support files with the
// 'supportFile' configuration option.
//
// You can read more here:
// https://on.cypress.io/configuration
// ***********************************************************

// Import commands.js using ES2015 syntax:
import './commands'

// Alternatively you can use CommonJS syntax:
// require('./commands')

Cypress.Commands.add('login', () => {
    beforeEach(() => {
		cy.request({
			url: `http://localhost:1234/auth/login`,
			method: "POST",
			credentials: "include",
			headers: new Headers({
				"Content-Type": "application/json",
			}),
			body: JSON.stringify({
				username: "grapluser",
				password: "graplpassword",
			}),
		}).then((body) => {
			const grapl_jwt = { user: { authenticationData: { token: body.token } } };
			window.localStorage.setItem("grapl_jwt", JSON.stringify(grapl_jwt));
		});
	});
})