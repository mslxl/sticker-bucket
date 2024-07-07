import InfoView from "@/components/info-view";
import TagViewer from "@/components/tag-viewer";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Sticker, StickerImg, StickerText } from "@/lib/cmd/library";
import { convertFileSrc } from "@tauri-apps/api/core";
import clsx from "clsx";
import { memo } from "react";
import { LuArrowLeft } from "react-icons/lu";
import { useLoaderData } from "react-router-dom";
import * as shell from "@tauri-apps/plugin-shell";
import { useDialog } from "@/lib/hook/useDialog";

const StickerImgOverview = memo(
  ({ className, sticker }: { className?: string; sticker: StickerImg }) => {
    const url = convertFileSrc(sticker.path);

    return (
      <div className={clsx("space-y-2", className)}>
        <div className="flex items-center justify-center">
          <img src={url} className="rounded-md border" />
        </div>
        <h4 className="text-muted-foreground leading-none whitespace-nowrap overflow-hidden text-ellipsis text-center">
          {sticker.name}
        </h4>
      </div>
    );
  }
);

function StickerTextOverview({
  className,
  sticker,
}: {
  className?: string;
  sticker: StickerText;
}) {
  return (
    <div
      className={clsx("whitespace-pre-line border p-4 rounded-md", className)}
    >
      {sticker.name}
    </div>
  );
}

function StickerOverview({
  className,
  sticker,
}: {
  className?: string;
  sticker: Sticker;
}) {
  if (sticker.ty == "PIC") {
    return (
      <StickerImgOverview className={className} sticker={sticker as StickerImg} />
    );
  } else if (sticker.ty == "TEXT") {
    return (
      <StickerTextOverview className={className} sticker={sticker as StickerText} />
    );
  } else {
    return (
      <InfoView
        title={`Unrecongized type of sticker: ${sticker.ty} (id: ${sticker.id})`}
        description="Please check the database"
      />
    );
  }
}

export default function StickerView() {
  const { sticker }: { sticker: Sticker } = useLoaderData() as any;

  const dialog = useDialog()
  async function openImageFile(){
    
    const path = (sticker as StickerImg).path
    const contin = await dialog.ask(`Open ${path} with default applications?`)
    if(!contin) return
    shell.open(path, 'xdg-open')

  }


  return (
    <div>
      <div className="h-16 border-b flex items-center space-x-2 p-2">
        <Button variant="ghost" onClick={() => history.back()}>
          <LuArrowLeft />
        </Button>
        <h4 className="overflow-hidden whitespace-nowrap text-ellipsis">
          {sticker.name}
        </h4>
      </div>
      <div className="p-8 space-y-2 grid grid-cols-1 md:grid-cols-2 gap-2">
        <StickerOverview sticker={sticker} />
        <div className="border p-8 space-y-2">
          <div className="space-y-0.5">
            <h4 className="font-medium leading-none">Package</h4>
            <p className="text-muted-foreground">{sticker.package}</p>
          </div>
          <Separator />
          <TagViewer tags={sticker.tags} />
        </div>
      </div>
      {sticker.ty == "PIC" ? (
        <p className="p-8 text-muted-foreground">
          Location: <button className="underline hover:underline-offset-2" onClick={openImageFile}>{(sticker as StickerImg).path}</button>
        </p>
      ) : null}
    </div>
  );
}
