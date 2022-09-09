import React from "react";
import { render, screen } from "@testing-library/react";
import App from "./App";

test("renders grapl welcome", () => {
  render(<App />);
  const welcomeElement = screen.getByText(/A Graph Analytics Platform for Detection and Response/i);
  expect(welcomeElement).toBeInTheDocument();
});
