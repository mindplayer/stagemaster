import { useEffect, useState } from "react";
import { CaretDownIcon } from "@phosphor-icons/react";
import { ResourceSearchDialog } from "./ResourceSearchDialog";
import type { ResourceOption } from "./resource-search";
import "./resource-picker.css";

export function ResourcePicker({
  label,
  placeholder,
  value,
  options,
  disabled,
  onSelect,
}: {
  label: string;
  placeholder: string;
  value?: string;
  options: ResourceOption[];
  disabled: boolean;
  onSelect(id: string): void;
}) {
  const [open, setOpen] = useState(false);
  const [intent, setIntent] = useState<{ id: string } | null>(null);
  const selected = options.find((option) => option.id === value);
  // Unmount the dialog before the application validates drafts: an invalid field
  // must be focusable, and dialog cleanup must not steal that error focus back.
  useEffect(() => {
    if (!intent) return;
    setIntent(null);
    if (!disabled && options.some((option) => option.id === intent.id))
      onSelect(intent.id);
  }, [intent, disabled, options, onSelect]);
  return (
    <>
      <button
        className="resource-picker-trigger"
        type="button"
        aria-label={label}
        aria-haspopup="dialog"
        aria-expanded={open}
        disabled={disabled || !options.length}
        title={selected?.label ?? placeholder}
        onClick={(e) => {
          e.currentTarget.focus();
          setOpen(true);
        }}
      >
        <span>{selected?.label ?? placeholder}</span>
        <CaretDownIcon aria-hidden="true" />
      </button>
      {open && (
        <ResourceSearchDialog
          label={label}
          value={value}
          options={options}
          onClose={() => setOpen(false)}
          onSelect={(id) => {
            setOpen(false);
            setIntent({ id });
          }}
        />
      )}
    </>
  );
}
