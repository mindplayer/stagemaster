import {
  withMotion,
  channelLabels,
  type ProfileDraft,
} from "../../fixture-tools";
export function ProfileMotionFields({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <>
      <label className="profile-motion-toggle">
        <input
          type="checkbox"
          checked={Boolean(value.positioning)}
          onChange={(e) => setDraft(withMotion(value, e.target.checked))}
        />
        两轴摇头灯
      </label>
      {value.positioning && (
        <section className="profile-motion">
          <h3>轴行程与输出方向</h3>
          <p className="wb-dim">
            按厂家通道表填写物理角度。零角光束沿灯具局部下方；安装朝向在舞台设置。此模型仅支持相交正交两轴。
          </p>
          {(["pan", "tilt"] as const).map((axis) => (
            <div key={axis} className="profile-meta">
              {(["minDegrees", "maxDegrees"] as const).map((key) => (
                <label key={key}>
                  {channelLabels[axis]}
                  {key === "minDegrees" ? "最小" : "最大"}
                  角度（°）
                  <input
                    name={`${axis}-${key}`}
                    aria-label={`${channelLabels[axis]}${key === "minDegrees" ? "最小" : "最大"}角度`}
                    value={value.positioning![axis][key]}
                    onChange={(e) =>
                      setDraft({
                        ...value,
                        positioning: {
                          ...value.positioning!,
                          [axis]: {
                            ...value.positioning![axis],
                            [key]: e.target.value,
                          },
                        },
                      })
                    }
                  />
                </label>
              ))}
              <label>
                {channelLabels[axis]}输出映射
                <select
                  aria-label={`${channelLabels[axis]}输出映射`}
                  value={
                    value.positioning![axis].reversed ? "reverse" : "forward"
                  }
                  onChange={(e) =>
                    setDraft({
                      ...value,
                      positioning: {
                        ...value.positioning!,
                        [axis]: {
                          ...value.positioning![axis],
                          reversed: e.target.value === "reverse",
                        },
                      },
                    })
                  }
                >
                  <option value="forward">低值 → 最小角度</option>
                  <option value="reverse">低值 → 最大角度</option>
                </select>
              </label>
            </div>
          ))}
        </section>
      )}
    </>
  );
}
