import { createSignal, For, Show, type JSX } from "solid-js";

/**
 * Props accepted by the {@link TagInput} component.
 */
export type TagInputProps = {
  /** Controlled list of tags (strings). */
  tags: string[];
  /** Callback fired whenever the tag list changes. */
  onChange: (newTags: string[]) => void;
  /** Accessible label rendered above the input. */
  label: string;
  /** Accessible description. */
  description?: string;
  /** Placeholder text shown inside the text input. */
  placeholder?: string;
  /** Error message displayed below the input. */
  error?: string;
  /** Maximum number of tags allowed (default: unlimited). */
  maxTags?: number;
};

/**
 * A keyboard-driven tag / chip input built with Tailwind CSS.
 *
 * ## Interaction
 * - Type a value and press **Enter** or **,** to commit the tag.
 * - Duplicate tags are silently ignored.
 * - When `maxTags` is reached, further additions are prevented and
 *   the error message (if provided) is displayed.
 * - Each chip has a **&times;** button to remove it.
 *
 * @param props - See {@link TagInputProps}.
 * @returns A JSX element representing the tag input.
 */
export default function TagInput(props: TagInputProps): JSX.Element {
  const [inputValue, setInputValue] = createSignal("");
  const [showError, setShowError] = createSignal(false);

  /**
   * Attempts to add a tag to the current list.
   *
   * @param rawValue - The raw string value to add as a tag.
   */
  const addTag = (rawValue: string) => {
    const trimmed = rawValue.trim();
    if (trimmed.length === 0) {
      return;
    }

    // Enforce the maximum-tag limit when specified.
    if (props.maxTags !== undefined && props.tags.length >= props.maxTags) {
      setShowError(true);
      return;
    }

    // Prevent duplicate entries.
    if (props.tags.includes(trimmed)) {
      setInputValue("");
      return;
    }

    props.onChange([...props.tags, trimmed]);
    setInputValue("");
    setShowError(false);
  };

  /**
   * Handles keydown events on the text input.
   *
   * - **Enter** or **,** – commit the tag and clear the input.
   * - **Backspace** on an empty input – remove the last tag.
   *
   * @param event - The keyboard event.
   */
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Enter" || event.key === ",") {
      event.preventDefault();
      addTag(inputValue());
      return;
    }

    if (
      event.key === "Backspace" &&
      inputValue() === "" &&
      props.tags.length > 0
    ) {
      const updated = props.tags.slice(0, -1);
      props.onChange(updated);
      setShowError(false);
    }
  };

  /**
   * Removes a tag at the specified index.
   *
   * @param index - Zero-based index of the tag to remove.
   */
  const removeTag = (index: number) => {
    const updated = props.tags.filter((_, i) => i !== index);
    props.onChange(updated);
    setShowError(false);
  };

  return (
    <div>
      <label
        for="tag-input-field"
        class="mb-1.5 block text-md font-medium text-foreground"
      >
        {props.label}
      </label>

      <Show when={props.description}>
        <label class="mb-1.5 block text-sm">{props.description}</label>
      </Show>
      <div class="flex min-h-[42px] flex-wrap items-center gap-1.5 rounded-md border border-input bg-background px-3 py-1.5 text-sm ring-offset-background transition-shadow focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2">
        <For each={props.tags}>
          {(tag, index) => (
            <span class="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
              <span>{tag}</span>
              <button
                type="button"
                aria-label={`Remove ${tag}`}
                onClick={() => removeTag(index())}
                class="ml-0.5 inline-flex size-4 items-center justify-center rounded-full text-primary/70 hover:bg-primary/20 hover:text-primary focus:outline-none"
              >
                &times;
              </button>
            </span>
          )}
        </For>
        <input
          id="tag-input-field"
          type="text"
          value={inputValue()}
          onInput={(e) => setInputValue(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder={props.placeholder ?? "Type and press Enter to add tags"}
          disabled={
            props.maxTags !== undefined && props.tags.length >= props.maxTags
          }
          aria-invalid={showError() && !!props.error}
          class="min-w-[120px] flex-1 border-none bg-transparent text-sm text-foreground placeholder-muted-foreground outline-none disabled:cursor-not-allowed disabled:opacity-50"
        />
      </div>
      {(showError() ||
        (props.maxTags !== undefined && props.tags.length >= props.maxTags)) &&
        props.error && (
          <p class="mt-1.5 text-xs text-red-500" role="alert">
            {props.error}
          </p>
        )}
    </div>
  );
}
