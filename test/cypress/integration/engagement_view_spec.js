describe('lenses', () => {
  it('loads a lens', () => {
    cy.login();
    
    cy.contains(/engagement/i).click();

    // upload analyzer
    cy.fixture('./upload_analyzer.sh')
      .then(
        (analyzer_src) => {
          cy.exec(analyzer_src);
        }
      );




    // // assert no login cookie

    // // click 'LOGIN' button
    // cy.contains(/login/i).click()

    // // wait for page change
    // cy.location('href').should('include', '/login');

    // // enter username
    // cy.get("[placeholder='Username']").type('grapluser') // known good demo password

    // // enter password
    // cy.get("[placeholder='Password']").type('graplpassword') // known good demo password

    // // click 'SUBMIT' button
    // cy.contains(/submit/i).click()

    // // assert login cookie exists
  })

  /*
  it('does not allow the user to log in with an invalid username or password', () => {
    cy.visit('http://grapl-engagement-view:1234')

    // click 'LOGIN' button
    var login_button = cy.contains(/login/i)
    login_button.click()

  })
  */
})

