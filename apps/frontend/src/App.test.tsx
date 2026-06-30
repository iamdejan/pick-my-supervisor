import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@solidjs/testing-library";
import App from "./App";

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Stub localStorage so theme persistence works in JSDOM.
    const store: Record<string, string> = {};
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(
      (key: string) => store[key] ?? null,
    );
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(
      (key: string, value: string) => {
        store[key] = value;
      },
    );
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the page heading", () => {
    render(() => <App />);
    expect(screen.getByText("Pick My Supervisor")).toBeInTheDocument();
  });

  it("renders the tag input with its label", () => {
    render(() => <App />);
    expect(screen.getByText("My interests")).toBeInTheDocument();
  });

  it("renders the textarea with 500 character limit", () => {
    render(() => <App />);
    const textarea = screen.getByPlaceholderText(
      "Describe your research background, preferred methodologies, and any specific requirements...",
    );
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveAttribute("maxLength", "500");
  });

  it("shows initial character count as 0 / 500", () => {
    render(() => <App />);
    expect(screen.getByText("0 / 500")).toBeInTheDocument();
  });

  it("updates character count when typing in the textarea", () => {
    render(() => <App />);
    const textarea = screen.getByPlaceholderText(
      "Describe your research background, preferred methodologies, and any specific requirements...",
    );
    fireEvent.input(textarea, {
      target: { value: "Hello World" },
    });
    expect(screen.getByText("11 / 500")).toBeInTheDocument();
  });

  it("renders the submit button", () => {
    render(() => <App />);
    expect(screen.getByText("Find My Supervisor")).toBeInTheDocument();
  });

  it("logs form data on submit", () => {
    const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
    render(() => <App />);

    const textarea = screen.getByPlaceholderText(
      "Describe your research background, preferred methodologies, and any specific requirements...",
    );
    fireEvent.input(textarea, {
      target: { value: "Looking for an ML expert" },
    });

    const submitBtn = screen.getByText("Find My Supervisor");
    fireEvent.click(submitBtn);

    expect(logSpy).toHaveBeenCalledWith("Form submitted:", {
      interests: [],
      description: "Looking for an ML expert",
    });
    logSpy.mockRestore();
  });

  it("toggles theme when the toggle button is clicked", () => {
    render(() => <App />);

    const toggleBtn = screen.getByLabelText("Switch to dark mode");
    expect(toggleBtn).toBeInTheDocument();

    fireEvent.click(toggleBtn);

    expect(screen.getByLabelText("Switch to light mode")).toBeInTheDocument();
  });
});
