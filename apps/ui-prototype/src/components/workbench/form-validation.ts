/** Keep browser constraint validation accessible while using product-language messages. */
export function validateEditorForm(
  form: HTMLFormElement | null,
  interactive = true,
): void {
  if (!form) throw new Error("编辑面板已关闭，请重新选择");
  for (const field of form.querySelectorAll<
    HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement
  >("input,select,textarea")) {
    field.setCustomValidity("");
    if (!field.willValidate) continue;
    const label = field.getAttribute("aria-label") ?? "此项";
    const validity = field.validity;
    let error = "";
    if (validity.valueMissing || (field.required && !field.value.trim()))
      error = `请填写${label}`;
    else if (validity.badInput) error = `${label}需要填写数字`;
    else if (validity.rangeUnderflow || validity.rangeOverflow)
      error = `${label}应在 ${field.getAttribute("min")}–${field.getAttribute("max")} 之间`;
    else if (validity.stepMismatch) {
      const step = field.getAttribute("step") ?? "1";
      error =
        step === "1"
          ? `${label}需要填写整数`
          : `${label}请按 ${step} 的步长输入`;
    } else if (validity.patternMismatch)
      error = `${label}${field.getAttribute("data-pattern-message") ?? "请使用 #RRGGBB 格式，例如 #3979FF"}`;
    else if (!validity.valid) error = `请检查${label}`;
    if (error) {
      field.setCustomValidity(error);
      if (interactive) field.reportValidity();
      throw new Error(error);
    }
  }
}
