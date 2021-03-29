describe("without authentication", () => {
    it("allows the user to log in with a valid username and password", () => {
        cy.getCookie("grapl_jwt").should("not.exist");
        cy.login_flow();
        cy.getCookie("grapl_jwt").should("exist");
    });
});

describe("with authentication", () => {
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
