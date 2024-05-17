import { useMemo, useState } from "react";
import { Button } from "../ui/button";
import { LuPlus, LuTrash } from "react-icons/lu";
import clsx from "clsx";
import * as z from "zod";
import { concat, contains, drop, take } from "lodash/fp";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../ui/alert-dialog";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormMessage,
} from "../ui/form";
import { Input } from "../ui/input";

interface PrefListProps {
  leading: string;
  description?: string;
  className?: string;
  items: string[];
  itemZod: z.ZodString;
  itemDescription?: string;
  onItemsChanged: (items: string[]) => void;
}
export default function PrefList({
  leading,
  description,
  className,
  items,
  itemZod,
  itemDescription,
  onItemsChanged,
}: PrefListProps) {
  const [dialogVisible, setDialogVisible] = useState(false);

  const schema = useMemo(
    () =>
      z.object({
        value: itemZod,
      }),
    [itemZod]
  );

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      value: "",
    },
  });

  function showItemAddDialog() {
    form.reset();
    setDialogVisible(true);
  }

  function deleteItem(index: number) {
    onItemsChanged(concat(take(index, items), drop(index + 1, items)));
  }
  function addItem(values: z.infer<typeof schema>) {
    if (!contains(values.value, items)) {
      onItemsChanged(concat(items, values.value));
    }
    setDialogVisible(false)
  }

  return (
    <>
      <div className={clsx("p-4", className)}>
        <div className="flex flex-row items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-sm font-medium leading-none">{leading}</h4>
            {description && (
              <p className="text-sm text-muted-foreground">{description}</p>
            )}
          </div>
          <Button variant="outline" onClick={showItemAddDialog}>
            <LuPlus />
          </Button>
        </div>
        <ul className="mt-4 overflow-y-auto">
          {items.map((v, index) => (
            <li
              className="p-4 border hover:bg-secondary flex flex-row items-center justify-between"
              key={v}
            >
              {v}
              <Button variant="ghost" onClick={() => deleteItem(index)}>
                <LuTrash />
              </Button>
            </li>
          ))}
        </ul>
      </div>
      <AlertDialog open={dialogVisible} onOpenChange={setDialogVisible}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Add Item</AlertDialogTitle>
            {description && (
              <AlertDialogDescription>{description}</AlertDialogDescription>
            )}
          </AlertDialogHeader>

          <Form {...form}>
            <form onSubmit={form.handleSubmit(addItem)} className="space-y-2">
              <FormField
                control={form.control}
                name="value"
                render={({ field }) => (
                  <FormItem>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    {itemDescription && (
                      <FormDescription>{itemDescription}</FormDescription>
                    )}
                    <FormMessage />
                  </FormItem>
                )}
              />
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <Button type="submit">Confirm</Button>
              </AlertDialogFooter>
            </form>
          </Form>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
