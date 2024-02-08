import {
  AlertDialogCancel,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import TextEditor from "./text-editor";
import { Button } from "./ui/button";
import { LuCheck, LuX } from "react-icons/lu";
import { useState } from "react";
import { Tag } from "./tag-viewer";
import { useDialog } from "@/lib/hook/useDialog";
import { createTextSticker } from "@/lib/cmd/library";

interface StickerFolderAddDialogProps {
  resolve: () => void;
  reject: () => void;
}

export default function StickerFolderAddDialog({resolve}: StickerFolderAddDialogProps) {
  const dialog = useDialog();
  const [content, setContent] = useState("");
  const [pkg, setPkg] = useState("");
  const [tags, setTag] = useState<Tag[]>([]);

  async function confirm() {
    if (content.trim().length == 0) {
      await dialog.message("Content can not be empty", { title: "Error" });
      return;
    }
    try {
      await createTextSticker(content, pkg, tags);
    } catch (e: any) {
      dialog.message(e, { title: "Error" });
      return
    }
    resolve()
  }

  return (
    <>
      <AlertDialogHeader className="flex flex-row items-center justify-end mb-1">
        <AlertDialogTitle>Add Text</AlertDialogTitle>
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
      <TextEditor
        content={content}
        onContentChanged={setContent}
        pkg={pkg}
        onPkgChanged={setPkg}
        tags={tags}
        onTagsChanged={setTag}
      />
    </>
  );
}
