import { FC, useEffect, useRef, useState } from "react";
import { Popover, PopoverContent, PopoverTrigger } from "./popover";
import { Button } from "./button";
import { LuCheck, LuChevronsUpDown } from "react-icons/lu";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./command";
import clsx from "clsx";
import { debounce, take, uniqBy } from "lodash/fp";

export interface ComboboxItemProps<T> {
  value: T | null;
}

export interface ComboboxProps<T> {
  suggest: (input: string) => Promise<T[]> | T[];
  keyOf: (value: T | null) => string;
  listItemRender: FC<ComboboxItemProps<T>>;
  comboboxRender: FC<ComboboxItemProps<T>>;
  inputPlaceholder?: string;
  suggestEmptyPlaceholder?: string;
  value?: T;
  allowEmpty?: boolean;
  onValueChanged: (value: T | null) => void;
}

export default function Combobox<T>({
  suggest,
  keyOf,
  allowEmpty,
  inputPlaceholder = "Input...",
  suggestEmptyPlaceholder = "Empty",
  comboboxRender: ComboboxRender,
  listItemRender: ListItemRender,
  value,
  onValueChanged: setValue,
}: ComboboxProps<T>) {
  const [open, setOpen] = useState(false);
  const [suggestItems, setSuggestItems] = useState<T[]>([]);

  const emitSuggest = useRef(0);
  const receivedSuggest = useRef(0);

  async function inputChanged(input: string) {
    let currentId = emitSuggest.current;
    emitSuggest.current++;
    const suggestResult = take(100, uniqBy(keyOf, await suggest(input)));
    // make only last query effect
    if (currentId > receivedSuggest.current) {
      setSuggestItems(suggestResult);
      receivedSuggest.current = currentId;
    }
  }
  const inputChangedDebounce = debounce(200, inputChanged);

  const [popoverWidth, setPopoverWidget] = useState(200);

  // initial default
  useEffect(() => {
    inputChanged("");
  }, []);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-full justify-between"
          onClick={(e) => {
            setPopoverWidget(e.currentTarget.offsetWidth);
          }}
        >
          <ComboboxRender value={value ?? null} />
          <LuChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="p-0" style={{width: popoverWidth}}>
        <Command>
          <CommandInput
            onValueChange={inputChangedDebounce}
            placeholder={inputPlaceholder}
          />
          <CommandList>
            <CommandEmpty>{suggestEmptyPlaceholder}</CommandEmpty>
            <CommandGroup>
              {suggestItems.map((item) => (
                <CommandItem
                  key={keyOf(item)}
                  value={keyOf(item)}
                  onSelect={(curValue) => {
                    if (allowEmpty) {
                      setValue(curValue == keyOf(value ?? null) ? null : item);
                    } else {
                      setValue(item);
                    }
                    setOpen(false);
                  }}
                >
                  <LuCheck
                    className={clsx(
                      "mr-2 h-4 w-4",
                      keyOf(value ?? null) == keyOf(item)
                        ? "opacity-100"
                        : "opacity-0"
                    )}
                  />
                  <ListItemRender value={item} />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
