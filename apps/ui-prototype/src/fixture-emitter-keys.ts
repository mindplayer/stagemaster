/** Exact attribute identity; this helper never evaluates light or maps DMX. */
export function splitEmitterAttribute(key: string) {
  const match = /^emitter\.([a-z][a-z0-9-]{0,31})\.([a-z-]+)$/.exec(key);
  return match ? { owner: match[1], attribute: match[2] } : null;
}
export function attributeBase(key: string) {
  return splitEmitterAttribute(key)?.attribute ?? key;
}
export function isEmitterFunction(key: string) {
  const split = splitEmitterAttribute(key);
  return (
    !!split &&
    ["shutter", "color-wheel", "gobo-wheel"].includes(split.attribute)
  );
}
