import { LuCheck, LuLock, LuSticker, LuX } from "react-icons/lu";
import TagViewer, { Tag, TagViewerProps, tagEq, tagNe } from "./tag-viewer";
import { info } from "@tauri-apps/plugin-log";
import {
  concat,
  filter,
  find,
  remove,
  uniq,
} from "lodash/fp";
import clsx from "clsx";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import * as z from "zod";
import { ZOD_TAG } from "@/const";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Form, FormControl, FormField, FormMessage } from "./ui/form";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./ui/tooltip";

const schema = z.object({
  value: ZOD_TAG,
});

interface TagEditorProps
  extends Omit<Omit<TagViewerProps, "valueRender">, "tags"> {
  onTagsChanged?: (tags: Tag[]) => void;
  lockable?: boolean;
  lockedTag?: Tag[];
  onLockedTagChanged?: (tag: Tag[]) => void;
  tags: Tag[];
}

export default function TagEditorA({
  tags,
  onTagsChanged,
  className,
  lockable,
  lockedTag,
  onLockedTagChanged,
  ...props
}: TagEditorProps) {
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      value: "",
    },
  });

  function removeTagItem(namespace: string, value: string) {
    info(`Remove tag item ${namespace}:${value}`);
    onTagsChanged && onTagsChanged(filter(tagNe({ namespace, value }), tags));
  }
  function addTagItem(values: z.infer<typeof schema>) {
    info(`Add tag item ${values.value}`);
    const [namespace, value] = values.value.split(":", 2);
    onTagsChanged && onTagsChanged(uniq(concat(tags, { namespace, value })));
    form.reset();
  }

  function tryLockTag(tag: Tag) {
    if (!lockable) return;
    if (isLocked(tag)) {
      info(`Unlock tag ${tag.namespace}:${tag.value}`);
      const lock = remove(tagEq(tag)) as unknown as Tag[];
      onLockedTagChanged && onLockedTagChanged(lock);
    } else {
      info(`Lock tag ${tag.namespace}:${tag.value}`);
      const lock = uniq(concat(tag, lockedTag ?? []));
      onLockedTagChanged && onLockedTagChanged(lock);
    }
  }
  function isLocked(tag: Tag): boolean {
    if (!lockable) return false;
    return find(tagEq(tag) as any, lockedTag ?? []) != undefined;
  }

  return (
    <div className={clsx("", className)}>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(addTagItem)}>
          <FormField
            control={form.control}
            name="value"
            render={({ field }) => (
              <div className="space-y-2">
                <div className="relative ml-auto flex-1 md:grow-0 flex gap-1">
                  <LuSticker className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                  <FormControl>
                    <Input
                      type="search"
                      placeholder="Type tag to append..."
                      className="flex-1 rounded-lg bg-background pl-8"
                      {...field}
                    />
                  </FormControl>
                  <Button variant="outline" type="submit">
                    <LuCheck />
                  </Button>
                </div>
                <FormMessage />
              </div>
            )}
          />
        </form>
      </Form>
      <TagViewer
        tags={tags}
        {...props}
        valueRender={(tag) => (
          <span className="flex items-center">
            {isLocked(tag) ? <LuLock className="mr-1" /> : null}
            <TooltipProvider>
              <Tooltip disableHoverableContent delayDuration={0}>
                <TooltipTrigger asChild>
                  <button onClick={() => tryLockTag(tag)}>{tag.value}</button>
                </TooltipTrigger>
                {!lockable ? null : (
                  <TooltipContent>Click to toggle the lock</TooltipContent>
                )}
              </Tooltip>
            </TooltipProvider>
            <button
              className="ml-1 rounded-md hover:bg-secondary"
              onClick={() => removeTagItem(tag.namespace, tag.value)}
            >
              <LuX />
            </button>
          </span>
        )}
      />
    </div>
  );
}
