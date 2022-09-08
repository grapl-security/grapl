import React from "react";
import { render, screen } from "@testing-library/react";
import App from "./App";

test("renders learn react link", () => {
  render(<App />);
  const welcomeElement= screen.getByText(/A Graph Analytics Platform for Detection and Response/i);
  expect(welcomeElement).toBeInTheDocument();
});
