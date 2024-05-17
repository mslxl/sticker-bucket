import { AlertDialog, AlertDialogContent } from "@/components/ui/alert-dialog";
import { atom, useAtomValue, useSetAtom } from "jotai";
import { concat, drop, take } from "lodash/fp";
import { FC,ReactNode } from "react";

interface ModalItem {
  node: ReactNode;
  key: number;
}

const modalStackCounterAtom = atom(0);
export const modalStackAtom = atom<ModalItem[]>([]);

export const pushModalAtom = atom(null, (get, set, node: ReactNode) => {
  const key = get(modalStackCounterAtom);
  set(modalStackCounterAtom, key + 1);
  set(modalStackAtom, concat({ node, key }, get(modalStackAtom)));
});

export const popModalAtom = atom(null, (get, set) => {
  set(modalStackAtom, drop(1, get(modalStackAtom)));
});


interface DialogProps{
    resolve: (data?: any)=>void
    reject: (err?: any)=>void
}
export const pushDialogAtom = atom(null, (_, set, dialog: FC<DialogProps>)=>{
    return new Promise((resolve, reject)=>{
        const Dialog = dialog
        set(pushModalAtom, <Dialog resolve={resolve} reject={reject}/>)
    })
})

const removeModalAtom = atom(null, (get, set, index: number) => {
  const old = get(modalStackAtom);
  set(modalStackAtom, concat(take(index, old), drop(index + 1, old)));
});

interface ModalStackProviderProps {
  children?: ReactNode;
}
export function ModalStackProvider({ children }: ModalStackProviderProps) {
  const stacks = useAtomValue(modalStackAtom);
  const removeModal = useSetAtom(removeModalAtom)

  return (
    <>
      {stacks.map((item, index) => (
        <AlertDialog key={item.key} open onOpenChange={()=>removeModal(index)}>
          <AlertDialogContent>{item.node}</AlertDialogContent>
        </AlertDialog>
      ))}
      {children}
    </>
  );
}
