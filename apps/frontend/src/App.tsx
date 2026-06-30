import {
  createResource,
  createSignal,
  For,
  Match,
  Show,
  Switch,
  type JSX,
} from "solid-js";
import TagInput from "./components/TagInput";
import ThemeToggle from "./components/ThemeToggle";
import { createTheme } from "./lib/use-theme";

type PickSupervisorRequest = {
  interesting_topics: string[];
  additional_text: string;
};

type PickSupervisorData = {
  id: number;
  name: string;
  slug: string;
};

type PickSupervisorResponse = {
  potential_supervisors?: PickSupervisorData[];
};

type ResourceParams = {
  tags: string[];
  description: string;
};

async function pickSupervisor(
  params: ResourceParams,
): Promise<PickSupervisorResponse> {
  const myHeaders = new Headers();
  myHeaders.append("Content-Type", "application/json");

  const request: PickSupervisorRequest = {
    interesting_topics: params.tags,
    additional_text: params.description,
  };
  const payload = JSON.stringify(request);

  const requestOptions: RequestInit = {
    method: "POST",
    headers: myHeaders,
    body: payload,
    redirect: "follow",
  };
  const response = await fetch(
    `${import.meta.env["VITE_BACKEND_BASE_URL"]}/supervisors/pick`,
    requestOptions,
  );
  return response.json() as Promise<PickSupervisorResponse>;
}

export default function App(): JSX.Element {
  const { theme, toggleTheme } = createTheme();

  const [tags, setTags] = createSignal<string[]>([]);
  const [description, setDescription] = createSignal("");
  const [charCount, setCharCount] = createSignal(0);
  const maxChars = 500;

  /**
   * Search params only get set on form submit so the resource fetcher is
   * not called on every keystroke. When the signal is undefined, the
   * resource is idle (no request sent).
   */
  const [searchParams, setSearchParams] = createSignal<ResourceParams>();

  const [pickSupervisorTask] = createResource(searchParams, pickSupervisor);

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
   * Triggers the pickSupervisor API call by setting the search-params
   * signal. Because the source is captured as a reactive value inside
   * createResource, changing it causes a re-fetch.
   */
  const handleSubmit = (event: Event) => {
    event.preventDefault();

    setSearchParams({
      tags: tags(),
      description: description(),
    });
  };

  /**
   * Builds the UMExpert profile URL from a lecturer slug.
   * @param slug - The unique slug identifier for the lecturer.
   * @returns The full UMExpert profile URL.
   */
  const buildUmExpertUrl = (slug: string): string =>
    `https://umexpert.um.edu.my/${slug}.html`;

  return (
    <div class="relative flex min-h-screen flex-col items-center justify-center bg-background text-foreground">
      {/* Theme toggle in the top-right corner */}
      <div class="absolute right-4 top-4">
        <ThemeToggle theme={theme} onToggle={toggleTheme} />
      </div>

      <main
        class="w-full max-w-lg rounded-lg border border-border bg-background p-6 shadow-sm sm:p-8"
        classList={{ "opacity-50": pickSupervisorTask.loading }}
      >
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
            disabled={pickSupervisorTask.loading}
            class="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
          >
            Find My Supervisor
          </button>
        </form>
      </main>

      {/* Results area — shown after a successful or failed fetch */}
      <Show
        when={pickSupervisorTask.state === "ready" || pickSupervisorTask.error}
      >
        <section class="mt-6 w-full max-w-lg" aria-live="polite">
          <Switch>
            <Match when={pickSupervisorTask.error}>
              <p class="rounded-md border border-red-200 bg-red-50 p-4 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-200">
                Something went wrong while fetching supervisors. Please try
                again.
              </p>
            </Match>

            <Match when={pickSupervisorTask()?.potential_supervisors?.length}>
              <div class="rounded-md border border-border p-4">
                <h2 class="mb-1 text-lg font-semibold text-foreground">
                  Yeay, we found you{" "}
                  {pickSupervisorTask()!.potential_supervisors!.length}{" "}
                  supervisor
                  {pickSupervisorTask()!.potential_supervisors!.length !== 1
                    ? "s"
                    : ""}
                  :
                </h2>
                <p class="mb-3 text-sm text-muted-foreground">
                  You can click the link on supervisor's name to find out more
                  about them.
                </p>
                <ul class="space-y-2">
                  <For each={pickSupervisorTask()!.potential_supervisors!}>
                    {(supervisor) => (
                      <li>
                        <a
                          href={buildUmExpertUrl(supervisor.slug)}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="text-primary underline-offset-2 hover:underline dark:text-foreground"
                        >
                          {supervisor.name}
                        </a>
                      </li>
                    )}
                  </For>
                </ul>
              </div>
            </Match>

            <Match when={!pickSupervisorTask()?.potential_supervisors?.length}>
              <p class="text-sm text-muted-foreground">
                No matching supervisors found. Try adjusting your interests or
                description.
              </p>
            </Match>
          </Switch>
        </section>
      </Show>

      <Show when={pickSupervisorTask.loading}>
        <div class="absolute inset-0 flex items-center justify-center bg-background/50">
          <div class="h-12 w-12 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      </Show>
    </div>
  );
}
