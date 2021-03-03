describe("sanity check", () => {
	it("passes", () => {
		expect(true).to.equal(true);
	});
});

describe("application loads", () => {
	it("visits the front page", () => {
		cy.visit("/");
	});
});

describe("login form test", () => {
	it("allows the user to log in with a valid username and password", () => {
		cy.visit("/");
		cy.contains(/login/i).click();
		cy.location("href").should("include", "/login");

		cy.get("[placeholder='Username']").type("grapluser"); // known good demo password
		cy.get("[placeholder='Password']").type("graplpassword"); // known good demo password
		cy.contains(/submit/i).click();
	});
});

describe("auth validation", () => {
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

		cy.contains("login").should("not.exist");
		it("checks we're authenticated on model-plugin page", () => {
			cy.get("grapl_jwt").should("exist");
			
		});

		it("uploads a model plugin and validates rendering in plugin table ", () => {
			cy.url().should("include", "plugins");
			cy.contains(/plugin/i).click();
			const filePath = "../fixtures/sample_plugins.zip";
			cy.get('input[type="file"]').attachFile(filePath);
			cy.get(".submitBtn").click();
			cy.contains("Successfully").should("exist");
			cy.contains("grapl_plug_ins").should("exist");
		})
	});
});
