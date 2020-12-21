import React, { useEffect, useState } from "react";
import Button from "@material-ui/core/Button";
import { useAsync } from "react-async-hook";
import { checkLogin } from "../Login";
import { Link } from "react-router-dom";
import { dasboardStyles } from "./makeStyles/DashboardStyles";
import { LoginNotification, GraplHeader } from "./reusableComponents";
import { getNotebookUrl } from "../services/notebookService";

const useStyles = dasboardStyles;

export default function Dashboard() {
  const asyncSagemakerUrl = useAsync(getNotebookUrl, []);
  const classes = useStyles();

  const [state, setState] = useState({
    loggedIn: true,
    renderedOnce: false,
  });

  useEffect(() => {
    if (state.renderedOnce) {
      return;
    }

    const interval = setInterval(async () => {
      await checkLogin().then((loggedIn) => {
        if (!loggedIn) {
          console.warn("Logged out");
        }
        setState({
          loggedIn: loggedIn || false,
          renderedOnce: true,
        });
      });
    }, 2000);

    return () => {
      clearInterval(interval);
    };
  }, [state, setState]);

  console.log("state - loggedin", state.loggedIn);

  const loggedIn = state.loggedIn;

  if (asyncSagemakerUrl.loading || !asyncSagemakerUrl.result) {
    return <div>Loading...</div>;
  }

  const openSagemakerUrl = () => window.open(asyncSagemakerUrl.result);

  return (
    <>
      <GraplHeader displayBtn={false} />

      <div className={classes.dashboard}>
        <section className={classes.nav}>
          <Link to="/engagements" className={classes.link}>
            Engagements
          </Link>
          <Link to="/plugins" className={classes.link}>
            Upload Plugin
          </Link>
          <Button onClick={openSagemakerUrl} className={classes.link}>
            Open Engagement Notebook
          </Button>
        </section>

        <section className={classes.welcome}>
          <div className={classes.loggedIn}>
            {!loggedIn ? <LoginNotification /> : ""}
          </div>

          <h1> Welcome! </h1>
        </section>
      </div>
    </>
  );
}
