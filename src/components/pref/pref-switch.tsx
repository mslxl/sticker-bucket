import { RefAttributes } from "react";
import { Switch } from "../ui/switch";
import { SwitchProps } from "@radix-ui/react-switch";

interface PrefSwitchProps extends RefAttributes<HTMLButtonElement>, SwitchProps {
  leading: string;
  description?: string;
}
export default function PrefSwitch({
  leading,
  description,
  ...props
}: PrefSwitchProps) {
  return (
    <div className="flex flex-row items-center justify-between p-4">
      <div className="space-y-0.5">
        <h4 className="text-sm font-medium leading-none">{leading}</h4>
        {description && (
          <p className="text-sm text-muted-foreground">{description}</p>
        )}
      </div>
      <Switch {...props} />
    </div>
  );
}
