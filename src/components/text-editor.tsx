import { forwardRef, useImperativeHandle, useState } from "react";
import PackageCombobox from "./package-combobox";
import TagEditor from "./tag-editor";
import { Label } from "./ui/label";
import { Textarea } from "./ui/textarea";
import { Toggle } from "./ui/toggle";
import { LuHeart, LuLock, LuUnlock } from "react-icons/lu";
import clsx from "clsx";
import { Tag } from "./tag-viewer";

interface TextEditorRef {
  get lockPackage(): boolean;
  set lockPackage(value: boolean);
  get lockedTags(): Tag[];
  set lockedTags(value: Tag[]);
  reset: () => void;
}

interface TextEditorProps {
  lockable?: boolean;
  fav?: boolean;
  onFavChanged?: (fav: boolean) => void;
  tags?: Tag[];
  onTagsChanged?: (tags: Tag[]) => void;
  content?: string;
  onContentChanged?: (content: string) => void;
  pkg?: string;
  onPkgChanged?: (pkg: string) => void;
}
export const TextEditor = forwardRef<TextEditorRef, TextEditorProps>(
  (
    {
      lockable,
      fav,
      onFavChanged,
      tags,
      onTagsChanged,
      content,
      onContentChanged,
      pkg,
      onPkgChanged,
    }: TextEditorProps,
    ref
  ) => {
    const [lockedTags, setLockedTags] = useState<Tag[]>([]);
    const [lockPackage, setLockPackage] = useState(false);

    function reset() {
      onContentChanged && onContentChanged("");
      onFavChanged && onFavChanged(false);
      if (lockable) {
        onTagsChanged && onTagsChanged(lockedTags);
      } else {
        onTagsChanged && onTagsChanged([]);
        onPkgChanged && onPkgChanged("");
      }
    }

    useImperativeHandle(ref, () => ({
      get lockPackage() {
        return lockPackage;
      },
      set lockPackage(value: boolean) {
        setLockPackage(value);
      },
      get lockedTags() {
        return lockedTags;
      },
      set lockedTags(value: Tag[]) {
        setLockedTags(value);
      },
      reset,
    }));

    return (
      <div className="grid items-center gap-2">
        <div className="flex flex-col space-y-1.5">
          <Label htmlFor="content">Content</Label>
          <div className="flex">
            <Textarea
              id="content"
              value={content}
              onChange={(e) =>
                onContentChanged && onContentChanged(e.target.value)
              }
            />
            <div className="flex flex-row">
              <Toggle pressed={fav} onPressedChange={onFavChanged}>
                <LuHeart className={clsx(fav ? "text-red-600" : null)} />
              </Toggle>
            </div>
          </div>
        </div>
        <div className="flex flex-col space-y-1.5">
          <Label htmlFor="package">Package</Label>
          <div className="space-x-2 flex">
            <PackageCombobox value={pkg} onValueChanged={onPkgChanged} />
            {!lockable ? null : (
              <Toggle pressed={lockPackage} onPressedChange={setLockPackage}>
                {lockPackage ? <LuLock /> : <LuUnlock />}
              </Toggle>
            )}
          </div>
        </div>
        <TagEditor
          tags={tags ?? []}
          onTagsChanged={onTagsChanged}
          lockable={lockable}
          lockedTag={lockedTags}
          onLockedTagChanged={setLockedTags}
        />
      </div>
    );
  }
);

export default TextEditor