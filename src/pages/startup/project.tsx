import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { LuMoreVertical, LuSearch } from "react-icons/lu";
import { useTranslation } from "react-i18next";
import style from "./index.module.sass";
import clsx from "clsx";

import * as dialog from "@tauri-apps/plugin-dialog";
import { useAtomValue, useSetAtom } from "jotai";
import {
  addPrefHistoryAtom,
  prefHistoryAtom,
  removePrefHistoryAtom,
} from "@/prefs";
import { capitalize, compose, identical, join, map, negate, take, takeLastWhile } from "lodash/fp";

export default function Project() {
  const { t } = useTranslation();

  async function openLibrary(path: string){
      addHistory(path);
  }


  const histories = useAtomValue(prefHistoryAtom);
  const addHistory = useSetAtom(addPrefHistoryAtom);
  const removeHistory = useSetAtom(removePrefHistoryAtom);

  const makeItemName = takeLastWhile(negate(identical('/')))
  const makeAvatorFallBack = compose([capitalize,join(""), take(2), makeItemName])

  const historiesList = map(
    (item) => (
      <li
        className={clsx(
          style.listItem,
          "p-2 rounded-md my-4 flex items-center space-x-4 transition-background-color hover:bg-accent hover:text-accent-foreground"
        )}
        onClick={()=> openLibrary(item)}
      >
        <Avatar>
          <AvatarFallback>{makeAvatorFallBack(item)}</AvatarFallback>
        </Avatar>
        <div className="flex-1 ">
          <p className="text-sm tracking-tight">{makeItemName(item)}</p>
          <p className="text-xs text-muted-foreground">{item}</p>
        </div>
        <Button
          className={clsx(style.menuBtn, "transition-opacity")}
          variant="outline"
        >
          <LuMoreVertical />
        </Button>
      </li>
    ),
    histories
  );


  async function onOpenLibraryBtnClick() {
    const path = await dialog.open({
      title: t("startup.btn.open"),
      directory: true,
      multiple: false,
    });
    if (path) {
      openLibrary(path)
    }
  }

  return (
    <>
      <div className="m-4 relative flex gap-2 items-center">
        <LuSearch className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input className="flex-1 h-9 pl-8" />
        <Button onClick={onOpenLibraryBtnClick}>{t("startup.btn.open")}</Button>
      </div>
      <Separator />
      <ul className="px-8 py-2 overflow-y">
        {historiesList}
      </ul>
    </>
  );
}
