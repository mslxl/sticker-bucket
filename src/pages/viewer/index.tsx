import InfoView from "@/components/info-view";
import TagViewer from "@/components/tag-viewer";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Sticky, StickyImg, StickyText } from "@/lib/cmd/library";
import { convertFileSrc } from "@tauri-apps/api/core";
import clsx from "clsx";
import { memo } from "react";
import { LuArrowLeft } from "react-icons/lu";
import { useLoaderData } from "react-router-dom";
import * as shell from "@tauri-apps/plugin-shell";
import { useDialog } from "@/lib/hook/useDialog";

const StickyImgOverview = memo(
  ({ className, sticky }: { className?: string; sticky: StickyImg }) => {
    const url = convertFileSrc(sticky.path);

    return (
      <div className={clsx("space-y-2", className)}>
        <div className="flex items-center justify-center">
          <img src={url} className="rounded-md border" />
        </div>
        <h4 className="text-muted-foreground leading-none whitespace-nowrap overflow-hidden text-ellipsis text-center">
          {sticky.name}
        </h4>
      </div>
    );
  }
);

function StickyTextOverview({
  className,
  sticky,
}: {
  className?: string;
  sticky: StickyText;
}) {
  return (
    <div
      className={clsx("whitespace-pre-line border p-4 rounded-md", className)}
    >
      {sticky.name}
    </div>
  );
}

function StickyOverview({
  className,
  sticky,
}: {
  className?: string;
  sticky: Sticky;
}) {
  if (sticky.ty == "PIC") {
    return (
      <StickyImgOverview className={className} sticky={sticky as StickyImg} />
    );
  } else if (sticky.ty == "TEXT") {
    return (
      <StickyTextOverview className={className} sticky={sticky as StickyText} />
    );
  } else {
    return (
      <InfoView
        title={`Unrecongized type of sticky: ${sticky.ty} (id: ${sticky.id})`}
        description="Please check the database"
      />
    );
  }
}

export default function StickyView() {
  const { sticky }: { sticky: Sticky } = useLoaderData() as any;

  const dialog = useDialog()
  async function openImageFile(){
    
    const path = (sticky as StickyImg).path
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
          {sticky.name}
        </h4>
      </div>
      <div className="p-8 space-y-2 grid grid-cols-1 md:grid-cols-2 gap-2">
        <StickyOverview sticky={sticky} />
        <div className="border p-8 space-y-2">
          <div className="space-y-0.5">
            <h4 className="font-medium leading-none">Package</h4>
            <p className="text-muted-foreground">{sticky.package}</p>
          </div>
          <Separator />
          <TagViewer tags={sticky.tags} />
        </div>
      </div>
      {sticky.ty == "PIC" ? (
        <p className="p-8 text-muted-foreground">
          Location: <button className="underline hover:underline-offset-2" onClick={openImageFile}>{(sticky as StickyImg).path}</button>
        </p>
      ) : null}
    </div>
  );
}
