export class FixtureFieldError extends Error {
  field: string;
  constructor(field: string, message: string) {
    super(message);
    this.field = field;
  }
}
export function fixtureInteger(
  value: string,
  min: number,
  max: number,
  field: string,
  label: string,
) {
  if (!/^\d+$/.test(value.trim()) || Number(value) < min || Number(value) > max)
    throw new FixtureFieldError(field, `${label}需要填写 ${min}–${max} 的整数`);
  return Number(value);
}
