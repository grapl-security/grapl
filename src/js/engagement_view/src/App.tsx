import React from "react";

import GraplRoutes from "./routes";
import TraceProvider from "./contexts/TraceContext";

export default function App() {
    console.log("Welcome to Grapl");
    return (
        <>
            <TraceProvider>
                <GraplRoutes />
            </TraceProvider>
        </>
    );
}
