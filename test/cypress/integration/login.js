describe("without authentication", () => {
    it("allows the user to log in with a valid username and password", () => {
        cy.getCookie("grapl_jwt").should("not.exist");
        cy.login_flow();
        cy.getCookie("grapl_jwt").should("exist");
    });
});

describe("with authentication", () => {
    // You will notice:
    //   Cypress.Cookies.preserveOnce('grapl_jwt');
    // is duplicated across `before` and `beforeEach`.
    // It's redundant, but it works.
    // We can experiment with removing it later.

    before(() => {
        cy.clearCookies();
        Cypress.Cookies.preserveOnce('grapl_jwt');
        cy.login_flow();
    });

    after(() => {
        cy.clearCookies();
    });

    beforeEach(() => {
        Cypress.Cookies.preserveOnce('grapl_jwt');
    });

    // the following three (seemingly no-op) tests demonstrate that an
    // authentication cookie was set on the browser in the `before` hook,
    // and that it is successfully preserved across separate tests:

    it("(1) checks to make sure grapl_jwt was set", () => {
        cy.getCookie("grapl_jwt").should("exist");
    })

    it("(2) checks to make sure grapl_jwt was set", () => {
        cy.getCookie("grapl_jwt").should("exist");
    })

    it("(3) checks to make sure grapl_jwt was set", () => {
        cy.getCookie("grapl_jwt").should("exist");
    })
});
