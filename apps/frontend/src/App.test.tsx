import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  cleanup,
  waitFor,
} from "@solidjs/testing-library";
import App from "./App";

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the page successfully", () => {
    render(() => <App />);
    expect(screen.getByText("Increment: 0")).toBeInTheDocument();
    expect(screen.getByText("Decrement: 0")).toBeInTheDocument();
  });

  it("increment the number when `Increment` button is clicked", async () => {
    render(() => <App />);
    const incrementBtn = screen.getByText(/Increment: /i);
    expect(incrementBtn).toBeInTheDocument();
    fireEvent.click(incrementBtn);
    await waitFor(() => {
      expect(incrementBtn.textContent).toBe("Increment: 1");
    });
  });

  it("decrement the number when `Decrement` button is clicked", async () => {
    render(() => <App />);
    const decrementBtn = screen.getByText(/Decrement: /i);
    expect(decrementBtn).toBeInTheDocument();
    fireEvent.click(decrementBtn);
    await waitFor(() => {
      expect(decrementBtn.textContent).toBe("Decrement: -1");
    });
  });
});
