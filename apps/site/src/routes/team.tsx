import { Avatar, AvatarFallback } from "@ui-sdk/react/avatar";

import { ContentSection } from "@/components/content-section";
import { PageHeader } from "@/components/page-header";
import { site, team } from "@/lib/site";

const milestones = [
  {
    detail:
      "Onboarding, document drafts, repository capture, persistence, history, refresh, export, and deletion work together.",
    id: "M1",
    status: "Current",
    title: "Onboarding and local context capture",
  },
  {
    detail:
      "TypeScript and Python symbols and references are indexed and queryable.",
    id: "M2",
    status: "Planned",
    title: "Code structure",
  },
  {
    detail:
      "Interactive agent work and assumptions connect to evidence-backed verdicts.",
    id: "M3",
    status: "Planned",
    title: "Assumption verification",
  },
  {
    detail: "Knowledge gaps produce optional learning interventions.",
    id: "M4",
    status: "Planned",
    title: "Learning loop",
  },
  {
    detail: "Study results and the thesis report are complete.",
    id: "M5",
    status: "Planned",
    title: "Evaluation",
  },
] as const;

const initials = (name: string) =>
  name
    .split(" ")
    .map((part) => part[0])
    .join("");

export const TeamPage = () => (
  <div>
    <PageHeader
      description="Coderune is a final-year research project at the University of Sri Jayewardenepura. The team combines application engineering, local storage, and research evaluation."
      title="Group 23, building a shared-context layer."
    >
      <div className="flex flex-wrap gap-4 text-sm">
        <a
          className="underline underline-offset-4"
          href={site.repositoryUrl}
          rel="noreferrer"
          target="_blank"
        >
          Repository
        </a>
        <a
          className="underline underline-offset-4"
          href={site.proposalUrl}
          rel="noreferrer"
          target="_blank"
        >
          Proposal
        </a>
      </div>
    </PageHeader>

    <ContentSection id="team" title="Team">
      <p className="max-w-[40rem]">
        Three responsibilities: application engineering, local storage and
        capture, and research evaluation.
      </p>
      <div className="mt-6 grid gap-3 lg:grid-cols-3">
        {team.map((member) => (
          <div
            className="bg-card rounded-[var(--radius-panel)] p-5"
            key={member.name}
          >
            <div className="flex items-baseline justify-between gap-3">
              <span className="text-muted-foreground text-xs">
                {member.index}
              </span>
              <Avatar className="team-avatar">
                <AvatarFallback>
                  <span aria-hidden="true">{initials(member.name)}</span>
                  <span className="visually-hidden">{member.name}</span>
                </AvatarFallback>
              </Avatar>
            </div>
            <h3 className="mt-4 text-sm font-medium">{member.name}</h3>
            <p className="text-muted-foreground mt-1 text-xs">{member.role}</p>
            <p className="text-muted-foreground mt-3 text-sm leading-6">
              {member.focus}
            </p>
          </div>
        ))}
      </div>

      <div className="bg-card mt-6 rounded-[var(--radius-panel)] p-5">
        <h3 className="text-sm font-medium">{site.university}</h3>
        <p className="text-muted-foreground mt-1 text-sm">
          {site.degree} · {site.group} · 2026
        </p>
        <p className="text-muted-foreground mt-3 max-w-[40rem] text-sm leading-6">
          Context-aware assumption verification and shared-context mechanism for
          AI coding assistants using workspace signals.
        </p>
      </div>

      <p className="mt-6 flex flex-wrap gap-4 text-sm">
        <a
          className="underline underline-offset-4"
          href={site.proposalUrl}
          rel="noreferrer"
          target="_blank"
        >
          Research proposal
        </a>
        <a
          className="underline underline-offset-4"
          href={site.presentationUrl}
          rel="noreferrer"
          target="_blank"
        >
          Presentation
        </a>
      </p>
      <p className="text-muted-foreground mt-2 max-w-[40rem] text-xs leading-5">
        The public documents describe the research plan. Implementation evidence
        and completion are tracked in the repository.
      </p>
    </ContentSection>

    <ContentSection id="milestones" title="Milestones">
      <p className="max-w-[40rem]">
        The project is organised around five milestones. Only the first is being
        built now.
      </p>
      <section aria-label="Project milestones" className="table-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Milestone</th>
              <th scope="col">Status</th>
              <th scope="col">Detail</th>
            </tr>
          </thead>
          <tbody>
            {milestones.map((milestone) => (
              <tr key={milestone.id}>
                <td>
                  {milestone.id} · {milestone.title}
                </td>
                <td
                  className={
                    milestone.status === "Current"
                      ? "text-foreground"
                      : "text-muted-foreground"
                  }
                  data-label="Status"
                >
                  {milestone.status}
                </td>
                <td className="text-muted-foreground" data-label="Detail">
                  {milestone.detail}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </ContentSection>
  </div>
);
