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

describe("authentication", () => {
	it("allows the user to log in with a valid username and password", () => {
		cy.visit("/");
		cy.contains(/login/i).click();
		cy.location("href").should("include", "/login");

		cy.get("[placeholder='Username']").type("grapluser"); // known good demo password
		cy.get("[placeholder='Password']").type("graplpassword"); // known good demo password
		cy.contains(/submit/i).click();
	});
});

describe("login test", () => {
	beforeEach(() => {
		cy.visit("/login");
		cy.location("href").should("include", "/login");
		cy.request({
			url: `http://localhost:1234/auth/login`,
			method: "POST",
			credentials: "include",
			hearders: new Headers({
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
	});
});

describe("checks for valid cookies", () => {
	it("checks for valid cookies", () => {
		cy.getCookie("grapl_jwt");
	});
});
