import { Button } from "@/components/ui/button";
import {
  LuArrowDown,
  LuArrowLeft,
  LuSkipBack,
  LuSkipForward,
} from "react-icons/lu";
import { useLoaderData } from "react-router-dom";
import { BatchAddStickyLoaderData } from "./data-loader";
import { useMemo, useRef, useState } from "react";
import { Progress } from "@/components/ui/progress";
import { convertFileSrc } from "@tauri-apps/api/core";
import BatchAddCompleted from "./completed";
import { Tag } from "@/components/tag-viewer";
import { useDialog } from "@/lib/hook/useDialog";
import StickyEditor, { StickyEditorRef } from "@/components/sticky-editor";
import * as fs from "@tauri-apps/plugin-fs";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  blacklistPath,
  createPictureSticky,
  hasStickyFile,
} from "@/lib/cmd/library";
import { error, info, trace } from "@tauri-apps/plugin-log";
import { unreachable } from "@/lib/sys";

export default function BatchAddSticky() {
  const { images } = useLoaderData() as BatchAddStickyLoaderData;
  const [imagesIndex, setImageIndex] = useState(0);
  const [batchAddedCnt, setBatchAddCnt] = useState(0);
  const [allowBackwardCount, setAllowBackwardCount] = useState(0);

  const imagePath = images && images[imagesIndex];
  const imageUrl = useMemo(
    () => imagePath && convertFileSrc(imagePath),
    [imagePath]
  );

  const [name, setName] = useState("");
  const [pkg, setPkg] = useState("Inbox");
  const [fav, setFav] = useState(false);
  const [tags, setTags] = useState<Tag[]>([]);
  const [del, setDel] = useState(false);
  const dialogHandle = useDialog();

  const editorRef = useRef<StickyEditorRef>(null);

  const progressInTitle = useMemo(
    () => `${imagesIndex + 1}/${images?.length ?? "NaN"}`,
    [images, imagesIndex]
  );

  if (!images || images?.length == 0 || !imageUrl) {
    return <BatchAddCompleted total={batchAddedCnt} />;
  }

  function skip() {
    setAllowBackwardCount((v) => v + 1);
    setImageIndex((v) => v + 1);
  }
  function backward() {
    setAllowBackwardCount((v) => v - 1);
    setImageIndex((v) => v - 1);
  }

  function valid(): boolean {
    if (!imagePath) {
      unreachable();
    }
    if (name.length == 0) {
      dialogHandle.message("Name must not be empty");
      return false;
    }
    if (pkg.trim().length == 0) {
      dialogHandle.message("Package must not be empty");
      return false;
    }
    return true;
  }

  function skipAndIgnore() {
    if (images && images[imagesIndex]) {
      dialogHandle
        .ask("Are you sure skip this image forever", {
          title: "Ignore Image",
        })
        .then((confirm) => {
          if (confirm) {
            info(`Ignore image ${imagePath}`);
            blacklistPath(imagePath!).catch((err) => {
              error(err);
              dialogHandle.message(err, {
                title: "Error",
              });
            });
            skip();
          } else {
            trace("User cancel ignore operation");
          }
        });
    }
  }

  function nextButton() {
    (async () => {
      if (!(await valid())) {
        return;
      }
      if (await hasStickyFile(imagePath!)) {
        const cont = await dialogHandle.ask(
          "Sticky file was included in library, continue to add it?",
          {
            title: "File Already Exist",
            okLabel: "Continue",
          }
        );
        if (!cont) return;
      }

      info(
        `Add sticky ${JSON.stringify({
          imagePath,
          name,
          pkg,
          fav,
          tags,
        })} in batch`
      );
      try {
        await createPictureSticky(name, pkg, imagePath!, tags);
        info("Add sticky successfully");
      } catch (reason: any) {
        error(reason);
        dialogHandle.message(reason, {
          title: "Error",
        });
        return;
      }
      if (del) {
        try {
          info("Remove original file");
          await fs.remove(imagePath!);
        } catch (reason: any) {
          error(reason);
          dialogHandle.message(reason, {
            title: "Warning",
          });
        }
      }
      setAllowBackwardCount(0);
      editorRef.current?.reset();
      setImageIndex((v) => v + 1);
      setBatchAddCnt((v) => v + 1);
    })();
  }

  return (
    <div>
      <div className="border-b h-16 p-2 flex items-center">
        <Button size="icon" variant="ghost" onClick={() => history.back()}>
          <LuArrowLeft className="mr" />
        </Button>
        <h4 className="text-lg font-semibold">Add Sticky From Folder</h4>
        <span className="pl-2 flex-1">({progressInTitle})</span>
        <div className="space-x-2 flex">
          <span className="rounded-md flex items-center">
            <Button
              variant="ghost"
              className="rounded-none rounded-l-md border"
              onClick={skip}
            >
              Skip
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  className="rounded-none px-0 w-5 rounded-r-md border"
                >
                  <LuArrowDown />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuItem onClick={skipAndIgnore}>
                  <LuSkipForward className="pr-2" />
                  Ignore this image
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={backward}
                  disabled={allowBackwardCount == 0}
                >
                  <LuSkipBack className="pr-2" />
                  Backward
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </span>
          <Button onClick={nextButton}>Next</Button>
        </div>
      </div>
      <Progress
        className="rounded-none h-2"
        getValueLabel={(v, max) => `${v}/${max}`}
        value={(imagesIndex / images?.length) * 100}
      />
      <div className="p-8">
        <h5 className="text-muted-foreground leading-none py-2 text-end">
          File: {imagePath}
        </h5>
        <div className="flex items-center space-x-2">
          <Checkbox
            id="delete"
            checked={del}
            onCheckedChange={(v) => setDel(v == true)}
          />
          <label
            htmlFor="delete"
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Delete original file
          </label>
        </div>
        <StickyEditor
          ref={editorRef}
          name={name}
          pkg={pkg}
          fav={fav}
          tags={tags}
          onNameChanged={setName}
          onFavChanged={setFav}
          onPkgChanged={setPkg}
          onTagsChanged={setTags}
          className="p-4"
          path={imagePath}
          lockable
        />
      </div>
    </div>
  );
}
