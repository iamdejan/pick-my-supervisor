import type { JSX } from "solid-js";

export type SupervisorCardData = {
  name: string;
  slug: string;
  brief_summary: string;
};

/**
 * Builds the UMExpert profile URL from a lecturer slug.
 *
 * @param slug - The unique slug identifier for the lecturer.
 * @returns The full UMExpert profile URL.
 */
function buildUmExpertUrl(slug: string): string {
  return `https://umexpert.um.edu.my/${slug}.html`;
}

/**
 * Renders a card displaying a lecturer's name (linked to their UMExpert
 * profile) and a brief summary combining their biography and area of
 * expertise.
 *
 * @param props.name - The lecturer's full name.
 * @param props.slug - The slug used to build the UMExpert profile link.
 * @param props.brief_summary - A concise paragraph about the lecturer.
 */
export default function SupervisorCard(props: SupervisorCardData): JSX.Element {
  return (
    <article class="rounded-lg border border-border bg-background p-4 shadow-sm transition-shadow hover:shadow-md">
      <h3 class="mb-2 text-base font-semibold text-foreground">
        <a
          href={buildUmExpertUrl(props.slug)}
          target="_blank"
          rel="noopener noreferrer"
          class="text-primary underline-offset-2 hover:underline"
        >
          {props.name}
        </a>
      </h3>
      <p class="text-sm leading-relaxed text-muted-foreground">
        {props.brief_summary}
      </p>
    </article>
  );
}
