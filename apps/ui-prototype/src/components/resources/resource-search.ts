export interface ResourceOption {
  id: string;
  label: string;
  detail?: string;
  keywords?: string;
}
export function searchResources(options: ResourceOption[], query: string) {
  const normalize = (value: string) =>
    value.normalize("NFKC").toLocaleLowerCase();
  const terms = normalize(query).trim().split(/\s+/).filter(Boolean);
  return options.filter((option) => {
    const text = normalize(
      `${option.label} ${option.detail ?? ""} ${option.keywords ?? ""}`,
    );
    return terms.every((term) => text.includes(term));
  });
}
