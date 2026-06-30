import { render, screen, fireEvent, cleanup } from "@solidjs/testing-library";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import TagInput from "./TagInput";

describe("TagInput", () => {
  let onChange: (newTags: string[]) => void;

  beforeEach(() => {
    onChange = vi.fn();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the label and placeholder", () => {
    render(() => <TagInput tags={[]} onChange={onChange} label="Interests" />);
    expect(screen.getByText("Interests")).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText("Type and press Enter to add tags"),
    ).toBeInTheDocument();
  });

  it("adds a tag when pressing Enter", () => {
    render(() => <TagInput tags={[]} onChange={onChange} label="Interests" />);
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    fireEvent.input(input, { target: { value: "Rust" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith(["Rust"]);
  });

  it("adds a tag when pressing comma", () => {
    render(() => <TagInput tags={[]} onChange={onChange} label="Interests" />);
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    fireEvent.input(input, { target: { value: "Rust" } });
    fireEvent.keyDown(input, { key: "," });
    expect(onChange).toHaveBeenCalledWith(["Rust"]);
  });

  it("does not add empty tags", () => {
    render(() => <TagInput tags={[]} onChange={onChange} label="Interests" />);
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    fireEvent.input(input, { target: { value: "   " } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("does not add duplicate tags", () => {
    render(() => (
      <TagInput tags={["Rust"]} onChange={onChange} label="Interests" />
    ));
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    fireEvent.input(input, { target: { value: "Rust" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("removes a tag when clicking the remove button", () => {
    render(() => (
      <TagInput tags={["Rust"]} onChange={onChange} label="Interests" />
    ));
    const removeBtn = screen.getByLabelText("Remove Rust");
    fireEvent.click(removeBtn);
    expect(onChange).toHaveBeenCalledWith([]);
  });

  it("removes the last tag on Backspace when input is empty", () => {
    render(() => (
      <TagInput tags={["Rust"]} onChange={onChange} label="Interests" />
    ));
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    fireEvent.keyDown(input, { key: "Backspace" });
    expect(onChange).toHaveBeenCalledWith([]);
  });

  it("shows an error when maxTags is reached", () => {
    const maxTags = 2;
    render(() => (
      <TagInput
        tags={["Rust", "TypeScript"]}
        onChange={onChange}
        label="Interests"
        maxTags={maxTags}
        error="Too many tags"
      />
    ));
    expect(screen.getByRole("alert")).toHaveTextContent("Too many tags");
  });

  it("disables the input when maxTags is reached", () => {
    const maxTags = 2;
    render(() => (
      <TagInput
        tags={["Rust", "TypeScript"]}
        onChange={onChange}
        label="Interests"
        maxTags={maxTags}
        error="Too many tags"
      />
    ));
    const input = screen.getByPlaceholderText(
      "Type and press Enter to add tags",
    );
    expect(input).toBeDisabled();
  });
});
