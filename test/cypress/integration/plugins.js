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

    it("uploads a model plugin and validates rendering in plugin table ", () => {
        cy.contains(/plugin/i).click();
        cy.url().should("include", "plugins");
        cy.contains(/plugin/i).click();
        const filePath = "../fixtures/sample_plugins.zip";
        cy.get('input[type="file"]').attachFile(filePath);
        cy.get(".submitBtn").click();
        cy.contains("Successfully").should("exist");
        cy.contains("grapl_plug_ins").should("exist");
    });
});
