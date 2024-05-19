import { StickyThumb } from "@/lib/cmd/library";
import { StickyListLayoutProps } from ".";
import { convertFileSrc } from "@tauri-apps/api/core";
import clsx from "clsx";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";

interface GridItemProps {
  sticky: StickyThumb;
}

function GridItem({ sticky }: GridItemProps) {
  const url = convertFileSrc(sticky.path);
  return (
    <li>
      <Card className="h-full">
        <CardHeader className="h-full flex flex-col justify-end">
          <CardTitle className="flex justify-center flex-1">
            <div className="rounded-lg bg-secondary border flex items-center">
              <img src={url} />
            </div>
          </CardTitle>
          <Tooltip>
            <TooltipTrigger>
              <CardDescription className="whitespace-nowrap text-ellipsis overflow-hidden">
                {sticky.name}
              </CardDescription>
            </TooltipTrigger>
            <TooltipContent>{sticky.name}</TooltipContent>
          </Tooltip>
        </CardHeader>
      </Card>
    </li>
  );
}

export default function StickyGrid({
  stickies,
  className,
}: StickyListLayoutProps) {
  return (
    <div className={className}>
      <ul
        className={clsx(
          "grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-2"
        )}
      >
        <TooltipProvider>
          {stickies.map((s) => (
            <GridItem key={s.id} sticky={s} />
          ))}
        </TooltipProvider>
      </ul>
    </div>
  );
}
