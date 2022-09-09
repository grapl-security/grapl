// Our reportWebVitals file is created using with Create React App's bootstrapping process.
// This includes a performance relayer that allows for measuring and analyzing the performance
// of our application using different metrics.

// See: https://create-react-app.dev/docs/measuring-performance/#:%7E:text=reportWebVitals

import { ReportHandler } from "web-vitals";

const reportWebVitals = (onPerfEntry?: ReportHandler) => {
    if (onPerfEntry && onPerfEntry instanceof Function) {
        import("web-vitals").then(
            ({ getCLS, getFID, getFCP, getLCP, getTTFB }) => {
                getCLS(onPerfEntry);
                getFID(onPerfEntry);
                getFCP(onPerfEntry);
                getLCP(onPerfEntry);
                getTTFB(onPerfEntry);
            }
        );
    }
};

export default reportWebVitals;
