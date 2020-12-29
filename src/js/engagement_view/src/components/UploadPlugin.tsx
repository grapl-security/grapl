import React, { useEffect, useState } from "react";
import { GraplHeader } from "./reusableComponents";
import { useStyles } from "../utils/uploads/plugins/useStyles";
import { UploadForm } from "../utils/uploads/plugins/uploadPlugins";
import { PluginTable } from "../utils/uploads/plugins/pluginTable";
import { checkLogin } from "../services/loginService";
import { LoginNotification } from "./reusableComponents";

const UploadPlugin = () => {
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
          console.warn("Logged Out");
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

  const loggedIn = state.loggedIn;

  return (
    <>
      <GraplHeader displayBtn={true} />

      <div className={classes.loggedIn}>
        {!loggedIn ? <LoginNotification /> : ""}
      </div>

      <div className={classes.upload}>
        <div className={classes.uploadFormContainer}>
          <UploadForm />
          <div id="message" />
        </div>
        <div className={classes.pluginTable}>
          <PluginTable />
        </div>
      </div>
    </>
  );
};

export default UploadPlugin;
