describe('basic test', () => {
  it('passes', () => {
    expect(true).to.equal(true)
  })
})

describe('application loads', () => {
  it('visits the front page', () => {
    // set cypress.json's baseUrl: to this
    cy.visit('http://grapl-engagement-view:1234')
  })
})
