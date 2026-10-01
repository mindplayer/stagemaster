/** 只检查调用形状，不证明运行功能已实现。 */
import type { Meta } from "./shared-contracts";
import type {
  EffectBindingApi, EffectBindingRequest, EffectBindingReview, ColorRoleValue,
} from "./effect-library-contracts";

export async function useReviewedTemplate(
  api: EffectBindingApi, request: EffectBindingRequest, meta: Meta,
) {
  const reviewed = await api.review(request);
  if (!reviewed.ok || reviewed.value.status !== "ready") return reviewed;
  return api.apply({ ...meta, ticket: reviewed.value.ticket });
}

export function rejectedCalls(api: EffectBindingApi, review: EffectBindingReview, meta: Meta) {
  if (review.status !== "ready") {
    // @ts-expect-error 缺绑定或阻断没有可提交票据。
    void api.apply({ ...meta, ticket: review.ticket });
  }
  // @ts-expect-error 通道值不是可复用颜色角色。
  const raw: ColorRoleValue = { kind: "dmx", channel: 5, value: 14 };
  void raw;
}
