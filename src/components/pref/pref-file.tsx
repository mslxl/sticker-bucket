import { Button } from "../ui/button";
import { LuFile, LuInfo } from "react-icons/lu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
import * as dialog from "@tauri-apps/plugin-dialog";
import {
  OpenDialogOptions,
  SaveDialogOptions,
} from "@tauri-apps/plugin-dialog";
import { unreachable } from "@/lib/sys";
import { prop } from "lodash/fp";
import { FC } from "react";

interface PrefFileCommon {
  leading: string;
  description?: FC;
  value?: string;
}

type PrefFileProps = PrefFileCommon & DialogOption;

type DialogOption = OpenOption | SaveOption;

interface OpenOption {
  open?: true;
  option: OpenDialogOptions;
  onValueChange: (res: dialog.FileResponse) => void;
}
interface SaveOption {
  save?: true;
  option: SaveDialogOptions;
  onValueChange: (res: string) => void;
}

export default function PrefFile({
  leading,
  description: TooltipDescription,
  value,
  option,
  onValueChange,
  ...props
}: PrefFileProps) {
  function exploreFile() {
    (async () => {
      if (prop("open", props)) {
        const opt = option as OpenDialogOptions;
        const res = await dialog.open(opt);
        if (res) {
          onValueChange && onValueChange(res as any);
        }
      } else if (prop("save", props)) {
        const opt = option as SaveDialogOptions;
        const res = await dialog.save(opt);
        if (res) {
          onValueChange && onValueChange(res as any);
        }
      } else {
        await unreachable();
      }
    })();
  }

  return (
    <div className="flex flex-row items-center justify-between p-4">
      <div className="space-y-0.5">
        <h4 className="text-sm font-medium leading-none">
          {leading}

          {TooltipDescription && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger>
                  <LuInfo className="ml-1" />
                </TooltipTrigger>
                <TooltipContent>
                  <TooltipDescription />
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </h4>
        <p className="text-sm text-muted-foreground">{value}</p>
      </div>
      <Button onClick={exploreFile} variant="outline">
        <LuFile />
      </Button>
    </div>
  );
}
