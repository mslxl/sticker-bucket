import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { LuMoreVertical, LuSearch } from "react-icons/lu";
import style from "./index.module.sass";
import clsx from "clsx";

export default function Project() {

  return (
    <>
      <div className="m-4 relative flex gap-2 items-center">
        <LuSearch className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input className="flex-1 h-9 pl-8" />
        <Button>New Library</Button>
        <Button>Open</Button>
      </div>
      <Separator />
      <ul className="px-8 py-2 overflow-y">
        <li
          className={clsx(
            style.listItem,
            "p-2 rounded-md my-4 flex items-center space-x-4 transition-background-color hover:bg-accent hover:text-accent-foreground"
          )}
        >
          <Avatar>
            <AvatarFallback>Li</AvatarFallback>
          </Avatar>
          <div className="flex-1 ">
            <p className="text-sm tracking-tight">Lilies</p>
            <p className="text-xs text-muted-foreground">~/Pictures</p>
          </div>
          <Button
            className={clsx(style.menuBtn, "transition-opacity")}
            variant="outline"
          >
            <LuMoreVertical />
          </Button>
        </li>
      </ul>
    </>
  );
}
