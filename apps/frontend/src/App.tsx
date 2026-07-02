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
import SupervisorCard from "./components/SupervisorCard";
import { createTheme } from "./lib/use-theme";

type PickSupervisorRequest = {
  interesting_topics: string[];
  additional_text: string;
};

type PickSupervisorData = {
  name: string;
  slug: string;
  brief_summary: string;
};

type PickSupervisorResponse = {
  thinking?: string;
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
                {/* AI's thinking/reasoning info box with lightbulb icon */}
                <Show when={pickSupervisorTask()!.thinking}>
                  <div class="mb-3 flex gap-3 rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950">
                    <svg
                      class="mt-0.5 size-5 shrink-0 text-blue-600 dark:text-blue-400"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      viewBox="0 0 24 24"
                      aria-hidden="true"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M12 18v-5.25m0 0a6.01 6.01 0 0 0 1.5-.189m-1.5.189a6.01 6.01 0 0 1-1.5-.189m3.75 7.478a12.06 12.06 0 0 1-4.5 0m3.75 2.383a14.406 14.406 0 0 1-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 1 0-7.517 0c.85.493 1.509 1.333 1.509 2.316V18"
                      />
                    </svg>
                    <p class="text-sm text-blue-800 dark:text-blue-200">
                      {pickSupervisorTask()!.thinking}
                    </p>
                  </div>
                </Show>
                <p class="mb-3 text-sm text-muted-foreground">
                  You can click the link on supervisor's name to find out more
                  about them.
                </p>
                <div class="grid gap-3">
                  <For each={pickSupervisorTask()!.potential_supervisors!}>
                    {(supervisor) => (
                      <SupervisorCard
                        name={supervisor.name}
                        slug={supervisor.slug}
                        brief_summary={supervisor.brief_summary}
                      />
                    )}
                  </For>
                </div>
                {/* Disclaimer warning boxes */}
                <div class="mt-4 flex flex-col gap-2">
                  <div class="flex gap-3 rounded-md border border-amber-200 bg-amber-50 p-3 dark:border-amber-800 dark:bg-amber-950">
                    <svg
                      class="mt-0.5 size-5 shrink-0 text-amber-600 dark:text-amber-400"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      viewBox="0 0 24 24"
                      aria-hidden="true"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
                      />
                    </svg>
                    <p class="text-sm text-amber-800 dark:text-amber-200">
                      This data is scraped from UM Expert and may not reflect
                      the lecturer's current preferences or areas of interest.
                    </p>
                  </div>
                  <div class="flex gap-3 rounded-md border border-amber-200 bg-amber-50 p-3 dark:border-amber-800 dark:bg-amber-950">
                    <svg
                      class="mt-0.5 size-5 shrink-0 text-amber-600 dark:text-amber-400"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      viewBox="0 0 24 24"
                      aria-hidden="true"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
                      />
                    </svg>
                    <p class="text-sm text-amber-800 dark:text-amber-200">
                      AI may hallucinate and should not be trusted 100%. It is
                      the responsibility of students to verify and pick their
                      own supervisors.
                    </p>
                  </div>
                </div>
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
