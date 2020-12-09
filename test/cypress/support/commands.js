// ***********************************************
// This example commands.js shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
//
//
// -- This is a parent command --
// Cypress.Commands.add("login", (email, password) => { ... })
//
//
// -- This is a child command --
// Cypress.Commands.add("drag", { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add("dismiss", { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite("visit", (originalFn, url, options) => { ... })

Cypress.Commands.add("login", () => {
    cy.visit('/')

    // assert no login cookie

    // click 'LOGIN' button
    cy.contains(/login/i).click()

    // wait for page change
    cy.location('href').should('include', '/login');

    // enter username
    cy.get("[placeholder='Username']").type('grapluser') // known good demo password

    // enter password
    cy.get("[placeholder='Password']").type('graplpassword') // known good demo password

    // click 'SUBMIT' button
    cy.contains(/submit/i).click()

})