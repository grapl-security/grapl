import React from "react";
import logo from "./assets/graplLogoFull.svg";
import "./App.css";

function App() {
    return (
        <div className="App">
            <header className="App-header">
                <img src={logo} alt={"logo"} />
                <p>A Graph Analytics Platform for Detection and Response</p>
            </header>
        </div>
    );
}

export default App;
