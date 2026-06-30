import { createSignal, type JSX } from "solid-js";
import TagInput from "./components/TagInput";
import ThemeToggle from "./components/ThemeToggle";
import { createTheme } from "./lib/use-theme";

export default function App(): JSX.Element {
  const { theme, toggleTheme } = createTheme();

  const [tags, setTags] = createSignal<string[]>([]);
  const [description, setDescription] = createSignal("");
  const [charCount, setCharCount] = createSignal(0);
  const maxChars = 500;

  /**
   * Updates the description signal and the character count whenever the
   * user types in the textarea. Truncation beyond `maxChars` is
   * prevented via the `maxLength` attribute on the element itself.
   */
  const handleDescriptionInput = (
    event: Event & { currentTarget: HTMLTextAreaElement },
  ) => {
    const value = event.currentTarget.value;
    setDescription(value);
    setCharCount(value.length);
  };

  /**
   * Handles form submission. Currently logs the collected data to the
   * console for demonstration purposes.
   */
  const handleSubmit = (event: Event) => {
    event.preventDefault();

    console.log("Form submitted:", {
      interests: tags(),
      description: description(),
    });
  };

  return (
    <div class="relative flex min-h-screen flex-col items-center justify-center bg-background text-foreground">
      {/* Theme toggle in the top-right corner */}
      <div class="absolute right-4 top-4">
        <ThemeToggle theme={theme} onToggle={toggleTheme} />
      </div>

      <main class="w-full max-w-lg rounded-lg border border-border bg-background p-6 shadow-sm sm:p-8">
        <h1 class="mb-1 text-3xl font-bold tracking-tight text-foreground">
          Pick My Supervisor
        </h1>
        <p class="mb-6 text-md text-muted-foreground">
          Tell us about your research interests and what you are looking for in
          a supervisor.
        </p>

        <form onSubmit={handleSubmit} class="flex flex-col gap-5">
          <TagInput
            tags={tags()}
            description="Type your area of interest, e.g. 'Machine Learning' or 'HCI'. Press 'Enter' or ',' after you've finished typing the tag. Then, the tag will be added."
            onChange={setTags}
            label="My interests"
            placeholder="e.g. Machine Learning, HCI"
            maxTags={10}
            error="You can only add up to 10 tags."
          />

          <div>
            <label
              for="description"
              class="mb-1.5 block text-md font-medium text-foreground"
            >
              Tell us more about your supervisor's preferred background.
            </label>
            <textarea
              id="description"
              rows={5}
              maxLength={maxChars}
              value={description()}
              onInput={handleDescriptionInput}
              placeholder="Describe the supervisor's background that you desire, e.g. having research in a particular country."
              class="w-full resize-none rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder-muted-foreground ring-offset-background transition-shadow focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
            />
            <p
              class="mt-1 text-right text-xs text-muted-foreground"
              aria-live="polite"
            >
              {charCount()} / {maxChars}
            </p>
          </div>

          <button
            type="submit"
            class="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
          >
            Find My Supervisor
          </button>
        </form>
      </main>
    </div>
  );
}
