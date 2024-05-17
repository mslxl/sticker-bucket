import {
  AlertDialogAction,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { pushDialogAtom } from "@/store/modal";
import { MessageDialogOptions } from "@tauri-apps/plugin-dialog";
import { useSetAtom } from "jotai";

export function useDialog() {
  const pushDialog = useSetAtom(pushDialogAtom);

  return {
    message: (message: string, options?: MessageDialogOptions) => {
      pushDialog(() => (
        <>
          <AlertDialogHeader>
            <AlertDialogTitle>{options?.title ?? "Message"}</AlertDialogTitle>
          </AlertDialogHeader>
          <p>{message}</p>
          <AlertDialogFooter>
            <AlertDialogAction>{options?.okLabel ?? "OK"}</AlertDialogAction>
          </AlertDialogFooter>
        </>
      ));
    },
  };
}
