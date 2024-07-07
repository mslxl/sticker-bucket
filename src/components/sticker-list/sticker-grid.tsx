import { StickerImgThumb, StickerTextThumb, StickerThumb } from "@/lib/cmd/library";
import { StickerListLayoutProps } from ".";
import { convertFileSrc } from "@tauri-apps/api/core";
import clsx from "clsx";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
import { useNavigate } from "react-router-dom";

interface GridItemProps {
  sticker: StickerThumb;
}

function ImgItem({ sticker }: { sticker: StickerImgThumb }) {
  const url = convertFileSrc(sticker.path);
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
            {sticker.name}
          </CardDescription>
        </TooltipTrigger>
        <TooltipContent>{sticker.name}</TooltipContent>
      </Tooltip>
    </CardHeader>
  );
}
function TxtItem({ sticker }: { sticker: StickerTextThumb }) {
  return (
    <CardHeader className="h-full flex flex-col justify-end">
      <div className="overflow-hidden rounded-lg bg-secondary border p-2 max-h-60">
      <div className="leading-7 [&:not(:first-child)]:mt-6 text-ellipsis h-full overflow-hidden whitespace-pre-line">{sticker.name}</div>
      </div>
      <Tooltip>
        <TooltipTrigger>
          <CardDescription className="whitespace-nowrap text-ellipsis overflow-hidden">
            {sticker.name}
          </CardDescription>
        </TooltipTrigger>
        <TooltipContent>{sticker.name}</TooltipContent>
      </Tooltip>
    </CardHeader>
  );
}

function GridItem({ sticker }: GridItemProps) {
  const navgiate = useNavigate()
  return (
    <li>
      <Card className="h-full hover:bg-secondary" onClick={()=>navgiate(`/viewer/${sticker.id}`)}>
        {sticker.ty == "PIC" ? (
          <ImgItem sticker={sticker} />
        ) : (
          <TxtItem sticker={sticker} />
        )}
      </Card>
    </li>
  );
}

export default function StickerGrid({
  stickies,
  className,
}: StickerListLayoutProps) {
  return (
    <div className={className}>
      <ul
        className={clsx(
          "grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-2"
        )}
      >
        <TooltipProvider>
          {stickies.map((s) => (
            <GridItem key={s.id} sticker={s}/>
          ))}
        </TooltipProvider>
      </ul>
    </div>
  );
}
