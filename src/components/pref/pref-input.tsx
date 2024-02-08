import { useMemo, useState } from "react";
import { Input } from "../ui/input";
import { Button } from "../ui/button";
import { LuTextCursor } from "react-icons/lu";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
} from "../ui/alert-dialog";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import * as z from "zod";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormMessage,
} from "../ui/form";
interface PrefInputProps {
  leading: string;
  description?: string;
  inputDescription?: string;
  value?: string;
  onValueChange?: (value: string) => void;
  zod: z.ZodString;
}
export default function PrefInput({
  leading,
  description,
  value,
  inputDescription,
  onValueChange,
  zod,
}: PrefInputProps) {
  const [dialogVisible, setDialogVislble] = useState(false);

  const schema = useMemo(
    () =>
      z.object({
        value: zod,
      }),
    [zod]
  );

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      value: value,
    },
  });

  function showDialog() {
    form.reset();
    setDialogVislble(true);
  }
  function confirmInput(form: z.infer<typeof schema>) {
    setDialogVislble(false);
    onValueChange && onValueChange(form.value);
  }

  return (
    <>
      <div
        className="flex flex-row items-center justify-between p-4"
        onClick={showDialog}
      >
        <div className="space-y-0.5">
          <h4 className="text-sm font-medium leading-none">{leading}</h4>
          <p className="text-sm text-muted-foreground">{value}</p>
        </div>
        <Button>
          <LuTextCursor />
        </Button>
      </div>

      <AlertDialog open={dialogVisible} onOpenChange={setDialogVislble}>
        <AlertDialogContent>
          <Form {...form}>
            <form
              onSubmit={form.handleSubmit(confirmInput)}
              className="space-y-2"
            >
              <AlertDialogHeader>
                <AlertDialogHeader>{leading}</AlertDialogHeader>
                <AlertDialogDescription>{description}</AlertDialogDescription>
              </AlertDialogHeader>
              <FormField
                control={form.control}
                name="value"
                render={({ field }) => (
                  <FormItem>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    {inputDescription && (
                      <FormDescription>{inputDescription}</FormDescription>
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
