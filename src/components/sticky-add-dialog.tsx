import { useEffect, useRef, useState } from "react";
import * as dialog from "@tauri-apps/plugin-dialog";
import { Button } from "./ui/button";
import StickyEditor, { StickyEditorRef } from "./sticky-editor";
import { info } from "@tauri-apps/plugin-log";
import {
  AlertDialogCancel,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { LuCheck, LuX } from "react-icons/lu";
import { Tag } from "./tag-viewer";
import { useDialog } from "@/lib/hook/useDialog";

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

  const dialogHandle = useDialog()

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

  function valid():boolean{
    if(name.length == 0){
      dialogHandle.message("Name must not be empty")
      return false
    }
    if(pkg.trim().length == 0){
      dialogHandle.message("Package must not be empty")
      return false
    }
    return true
  }

  function confirm() {
    info(
      `Add sticky ${JSON.stringify({
        imagePath,
        name,
        pkg,
        fav,
        tags,
      })}`
    );
    ;(async()=>{
      if(!await valid()){
        return
      }
      editorRef.current?.reset();
    })()
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
