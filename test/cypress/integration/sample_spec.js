
describe('basic test', () => {
  it('passes', () => {
    expect(true).to.equal(true)
  })
})

describe('application loads', () => {
  it('visits the front page', () => {
    cy.visit('/')
  })
})

describe('authentication', () => {
  it('allows the user to log in with a valid username and password', () => {
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

    // assert login cookie exists
  })

  /*
  it('does not allow the user to log in with an invalid username or password', () => {
    cy.visit('/')

    // click 'LOGIN' button
    var login_button = cy.contains(/login/i)
    login_button.click()

  })
  */
})

