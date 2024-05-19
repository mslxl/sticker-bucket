import {
  ChangeEvent,
  FC,
  RefAttributes,
  forwardRef,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";
import { Input, InputProps } from "./input";
import { Popover, PopoverContent } from "./popover";
import { PopoverAnchor } from "@radix-ui/react-popover";

export interface ACItemProps<T> {
  value: T;
}

export interface ACProps<T>
  extends RefAttributes<HTMLInputElement>,
    InputProps {
  suggest: (input: string) => Promise<T[]> | T[];
  keyOf: (value: T) => string;
  listItemRender?: FC<ACItemProps<T>>;
  inputPlaceholder?: string;
}

export const StringAutoCompleteInput = withTypedAutoCompleteInput<string>();
export default StringAutoCompleteInput;

export function withTypedAutoCompleteInput<T>() {
  return forwardRef<HTMLInputElement, ACProps<T>>(
    (
      { suggest, keyOf, listItemRender, onBlur, onFocus, ...props }: ACProps<T>,
      ref
    ) => {
      const [open, setOpen] = useState(false);

      const txSeq = useRef(0);
      const rxSeq = useRef(0);

      const inputRef = useRef<HTMLInputElement>(null);
      useImperativeHandle(ref, () => inputRef.current!, [inputRef.current]);

      // set value of input, and then trigger the chagne event to notify other hook and auto-complete
      function setInputWithEvent(v: string) {
        Object.getOwnPropertyDescriptor(
          window.HTMLInputElement.prototype,
          "value"
        )!.set!.call(inputRef.current, v);
        inputRef.current?.dispatchEvent(
          new InputEvent("change", { bubbles: true, composed: true })
        );
      }

      const [suggestItems, setSuggestItems] = useState<T[]>([]);
      // set width of popover be equal with input
      const [componentWidth, setComponentWidth] = useState(0);

      function requestComplete() {
        setOpen(true);
        if (inputRef.current) {
          setComponentWidth(inputRef.current.offsetWidth);
        }

        txSeq.current++;
        const taskId = txSeq.current;

        (async () => {
          const result = await suggest(inputRef.current!.value);
          if (taskId < rxSeq.current) return;
          rxSeq.current = taskId;
          setSuggestItems(result);
        })();
      }

      function onInputFocus() {
        requestComplete();
      }


      function onInputBlur() {
        setOpen(false);
      }

      // use default item to render list
      const ListItem = useMemo(() => {
        return (
          listItemRender ??
          (({ value }: ACItemProps<T>) => (
            <div
              className="border-y p-2 hover:bg-secondary"
              onClick={() => {
                setInputWithEvent(keyOf(value));
              }}
            >
              {keyOf(value)}
            </div>
          ))
        );
      }, [listItemRender]);

      // forward event and make auto-complete
      function onInputUpdate(e: ChangeEvent<HTMLInputElement>) {
        props.onChange && props.onChange(e);
        requestComplete();
      }

      return (
        <Popover open={open && suggestItems.length > 0}>
          <PopoverAnchor asChild>
            <Input
              {...props}
              onChange={onInputUpdate}
              onBlur={(e) => {
                onInputBlur();
                onBlur && onBlur(e);
              }}
              onFocus={(e) => {
                onInputFocus();
                onFocus && onFocus(e);
              }}
              ref={inputRef}
            />
          </PopoverAnchor>
          <PopoverContent
            className="p-0"
            onOpenAutoFocus={(e) => e.preventDefault()}
            align="start"
            style={{ width: componentWidth }}
          >
            <ul>
              {suggestItems.map((item) => (
                <li key={keyOf(item)}>
                  <ListItem value={item} />
                </li>
              ))}
            </ul>
          </PopoverContent>
        </Popover>
      );
    }
  );
}
