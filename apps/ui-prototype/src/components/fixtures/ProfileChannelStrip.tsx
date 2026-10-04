import { profileChannelLabel, type ProfileDraft } from "../../fixture-tools";
export function ChannelStrip({ draft }: { draft: ProfileDraft }) {
  const width = Number(draft.footprint);
  if (!Number.isInteger(width) || width < 1 || width > 512) return null;
  const slots = Array.from({ length: width }, (_, i) =>
    draft.channels.flatMap((c) => [
      ...(Number(c.coarse) === i + 1
        ? [`${profileChannelLabel(draft, c.attribute)}粗调`]
        : []),
      ...(c.bits === "16" && Number(c.fine) === i + 1
        ? [`${profileChannelLabel(draft, c.attribute)}细调`]
        : []),
    ]),
  );
  return (
    <section className="profile-strip" aria-label="模式通道分布">
      <h3>通道分布</h3>
      <div>
        {slots.map((labels, i) => (
          <span
            key={i}
            className={
              labels.length > 1 ? "conflict" : labels.length ? "mapped" : ""
            }
            title={labels.join(" / ") || "未映射，输出 0"}
          >
            <b>{i + 1}</b>
            <small>{labels.join(" / ") || "空余"}</small>
          </span>
        ))}
      </div>
    </section>
  );
}
