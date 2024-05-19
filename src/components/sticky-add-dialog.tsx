import { useEffect, useRef, useState } from "react";
import * as dialog from "@tauri-apps/plugin-dialog";
import { Button } from "./ui/button";
import StickyEditor, { StickyEditorRef } from "./sticky-editor";
import { error, info } from "@tauri-apps/plugin-log";
import {
  AlertDialogCancel,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { LuCheck, LuX } from "react-icons/lu";
import { Tag } from "./tag-viewer";
import { useDialog } from "@/lib/hook/useDialog";
import { createSticky, hasStickyFile } from "@/lib/cmd/library";
import { unreachable } from "@/lib/sys";
import { Checkbox } from "./ui/checkbox";
import * as fs from "@tauri-apps/plugin-fs";

interface StickyAddDialogProps {
  resolve: () => void;
  reject: () => void;
}

export default function StickyAddDialog({ resolve }: StickyAddDialogProps) {
  const dialogVisible = useRef(false);
  const [imagePath, setImagePath] = useState<string>();
  const [name, setName] = useState("");
  const [pkg, setPkg] = useState("");
  const [fav, setFav] = useState(false);
  const [tags, setTags] = useState<Tag[]>([]);
  const [del, setDel] = useState(false);

  const dialogHandle = useDialog();

  const editorRef = useRef<StickyEditorRef>(null);

  useEffect(() => {
    if (!dialogVisible.current) {
      dialogVisible.current = true;
      dialog
        .open({
          title: "Select Sticky Image",
        })
        .then((res) => {
          info(`Select image ${res?.path}`);
          if (res) {
            setImagePath(res.path);
          } else {
            resolve();
          }
        })
        .finally(() => {
          dialogVisible.current = false;
        });
    }
  }, []);

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

  function confirm() {
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
        })}`
      );
      try {
        await createSticky(name, pkg, imagePath!, tags);
        info("Add sticky successfully")
      } catch (reason: any) {
        error(reason)
        dialogHandle.message(reason, {
          title: "Error",
        });
        return;
      }
      if (del) {
        try {
          info("Remove original file")
          await fs.remove(imagePath!);
        } catch (reason: any) {
          error(reason)
          dialogHandle.message(reason, {
            title: "Warning",
          });
        }
      }
      resolve();
    })();
  }

  return (
    <>
      <AlertDialogHeader className="flex flex-row items-center justify-end mb-1 border-b">
        <AlertDialogTitle>Add Sticky</AlertDialogTitle>
        <span className="flex-1" />
        <div>
          <AlertDialogCancel>
            <LuX />
          </AlertDialogCancel>
          <Button onClick={confirm}>
            <LuCheck />
          </Button>
        </div>
      </AlertDialogHeader>

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
        className="pt-1"
        path={imagePath}
      />
    </>
  );
}
