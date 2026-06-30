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
      "Describe the supervisor's background that you desire, e.g. having research in a particular country.",
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
      "Describe the supervisor's background that you desire, e.g. having research in a particular country.",
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

  it("renders the submit button and it is not disabled initially", () => {
    render(() => <App />);
    const submitBtn = screen.getByText("Find My Supervisor");
    expect(submitBtn).toBeInTheDocument();
    expect(submitBtn).not.toBeDisabled();
  });

  it("calls the API on form submit and displays results", async () => {
    const mockResponse = {
      potential_supervisors: [
        { id: 1, name: "DR. ZATI HAKIM BINTI AZIZUL HASAN", slug: "zati" },
        { id: 0, name: "DR. NARSIMLU KEMSARAM", slug: "narsimlu-kemsaram" },
      ],
    };

    globalThis.fetch = vi.fn().mockResolvedValueOnce({
      json: () => Promise.resolve(mockResponse),
    });

    render(() => <App />);

    const textarea = screen.getByPlaceholderText(
      "Describe the supervisor's background that you desire, e.g. having research in a particular country.",
    );
    fireEvent.input(textarea, {
      target: { value: "Looking for an ML expert" },
    });

    const submitBtn = screen.getByText("Find My Supervisor");
    fireEvent.click(submitBtn);

    expect(globalThis.fetch).toHaveBeenCalledWith(
      "http://127.0.0.1:8000/supervisors/pick",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          interesting_topics: [],
          additional_text: "Looking for an ML expert",
        }),
      }),
    );

    expect(
      await screen.findByText("DR. NARSIMLU KEMSARAM"),
    ).toBeInTheDocument();

    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(2);
    expect(links[0]).toHaveAttribute(
      "href",
      "https://umexpert.um.edu.my/zati.html",
    );
    expect(links[1]).toHaveAttribute(
      "href",
      "https://umexpert.um.edu.my/narsimlu-kemsaram.html",
    );
  });

  it("shows error message when the API call fails", async () => {
    globalThis.fetch = vi
      .fn()
      .mockRejectedValueOnce(new Error("Network error"));

    render(() => <App />);

    const submitBtn = screen.getByText("Find My Supervisor");
    fireEvent.click(submitBtn);

    expect(
      await screen.findByText(
        "Something went wrong while fetching supervisors. Please try again.",
      ),
    ).toBeInTheDocument();
  });

  it("shows empty message when no supervisors are returned", async () => {
    const mockResponse = { potential_supervisors: [] };

    globalThis.fetch = vi.fn().mockResolvedValueOnce({
      json: () => Promise.resolve(mockResponse),
    });

    render(() => <App />);

    const textarea = screen.getByPlaceholderText(
      "Describe the supervisor's background that you desire, e.g. having research in a particular country.",
    );
    fireEvent.input(textarea, {
      target: { value: "Looking for an ML expert" },
    });

    const submitBtn = screen.getByText("Find My Supervisor");
    fireEvent.click(submitBtn);

    expect(
      await screen.findByText(
        "No matching supervisors found. Try adjusting your interests or description.",
      ),
    ).toBeInTheDocument();
  });

  it("toggles theme when the toggle button is clicked", () => {
    render(() => <App />);

    const toggleBtn = screen.getByLabelText("Switch to dark mode");
    expect(toggleBtn).toBeInTheDocument();

    fireEvent.click(toggleBtn);

    expect(screen.getByLabelText("Switch to light mode")).toBeInTheDocument();
  });
});
