import { useMemo, useState } from "react";
import Combobox from "./ui/combobox";
import { concat, map } from "lodash/fp";
import { searchPackage } from "@/lib/cmd/library";

interface PackageComboboxItem {
  v: string;
  label: string;
}
interface PackageComboboxProps {
  onValueChanged?: (value: string) => void;
  value?: string;
}
export default function PackageCombobox({
  value,
  onValueChanged,
}: PackageComboboxProps) {
  const [label, setLabel] = useState("Inbox");
  const [innerValue, setInnerValue] = useState("Inbox");

  const finalItem = useMemo<PackageComboboxItem>(
    () => ({
      v: value && value.trim().length != 0 ? value : innerValue,
      label: value && value.trim().length != 0 && innerValue != value ? value! : label,
    }),
    [value, label, innerValue]
  );

  return (
    <Combobox
      suggest={async (input) => {
        let res = [{ v: "Inbox", label: "Inbox" }];
        if (input.trim().length > 0) {
          res.push({ v: input, label: `Create "${input}"` });
        }
        const localPkg = map(
          (name) => ({ v: name, label: name }),
          await searchPackage(input)
        );
        return concat(res, localPkg);
      }}
      comboboxRender={({ value }) => value?.label}
      listItemRender={({ value }) => <span>{value?.label}</span>}
      keyOf={(v) => v?.v ?? "__null"}
      value={finalItem}
      onValueChanged={(newValue) => {
        setLabel(newValue!.label);
        setInnerValue(newValue!.v);
        onValueChanged && onValueChanged(newValue!.v);
      }}
    />
  );
}
