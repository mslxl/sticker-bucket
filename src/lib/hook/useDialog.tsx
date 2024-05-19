import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { pushDialogAtom } from "@/store/modal";
import {
  ConfirmDialogOptions,
  MessageDialogOptions,
} from "@tauri-apps/plugin-dialog";
import { useSetAtom } from "jotai";

export function useDialog() {
  const pushDialog = useSetAtom(pushDialogAtom);

  return {
    message: (message: string, options?: MessageDialogOptions) => {
      return pushDialog(() => (
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
    ask: (message: string, options?: ConfirmDialogOptions) => {
      return pushDialog(({ resolve }) => (
        <>
          <AlertDialogHeader>
            <AlertDialogTitle>{options?.title ?? "Confirm"}</AlertDialogTitle>
          </AlertDialogHeader>
          <p>{message}</p>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => resolve(false)}>
              {options?.cancelLabel ?? "Cancel"}
            </AlertDialogCancel>
            <AlertDialogAction onClick={() => resolve(true)}>
              {options?.okLabel ?? "OK"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </>
      ));
    },
  };
}
