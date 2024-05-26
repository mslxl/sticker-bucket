import { StickyImg, StickyText, StickyThumb } from "@/lib/cmd/library";
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

function ImgItem({ sticky }: { sticky: StickyImg }) {
  const url = convertFileSrc(sticky.path);
  return (
    <CardHeader className="h-full flex flex-col justify-end">
      <CardTitle className="flex justify-center flex-1">
        <div className="rounded-lg bg-secondary border flex items-center">
          <img src={url} loading="lazy" />
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
  );
}
function TxtItem({ sticky }: { sticky: StickyText }) {
  return (
    <CardHeader className="h-full flex flex-col justify-end">
      <div className="overflow-hidden rounded-lg bg-secondary border p-2 max-h-60">
      <div className="leading-7 [&:not(:first-child)]:mt-6 text-ellipsis h-full overflow-hidden whitespace-pre-line">{sticky.name}</div>
      </div>
      <Tooltip>
        <TooltipTrigger>
          <CardDescription className="whitespace-nowrap text-ellipsis overflow-hidden">
            {sticky.name}
          </CardDescription>
        </TooltipTrigger>
        <TooltipContent>{sticky.name}</TooltipContent>
      </Tooltip>
    </CardHeader>
  );
}

function GridItem({ sticky }: GridItemProps) {
  return (
    <li>
      <Card className="h-full">
        {sticky.ty == "PIC" ? (
          <ImgItem sticky={sticky} />
        ) : (
          <TxtItem sticky={sticky} />
        )}
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
