import type { CheckIssue, ProjectCheck } from "./check-types.ts";
export function isCheckCurrent(
  check: ProjectCheck | null,
  projectId: string,
  generation: number,
  hasDrafts: boolean,
): boolean {
  return (
    !!check &&
    !hasDrafts &&
    check.generation === generation &&
    check.report.projectId === projectId
  );
}
export function filterIssues(
  issues: CheckIssue[],
  query: string,
  severity: "all" | "error" | "warning",
) {
  const words = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return issues.filter(
    (issue) =>
      (severity === "all" || issue.severity === severity) &&
      words.every((word) => issue.message.toLocaleLowerCase().includes(word)),
  );
}
