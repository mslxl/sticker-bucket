import { StickyThumb, searchSticky } from "@/lib/cmd/library";
import StickyGrid from "./sticky-grid";
import { useEffect, useRef, useState } from "react";
import { useAtomValue } from "jotai";
import cfg from "@/store/settings";
import { join, map } from "lodash/fp";
import InfoView from "../info-view";

export interface StickyListProps {
  stmt: string;
  className?: string;
  page?: number;
  onPageChange?: (page: number) => void;
}

export interface StickyListLayoutProps extends Omit<StickyListProps, "stmt"> {
  stickies: StickyThumb[];
}

export default function StickyList({ stmt, page, ...props }: StickyListProps) {
  const hideNSFW = useAtomValue(cfg.display.hideNsfw);
  const nsfwTags = useAtomValue(cfg.display.nsfwTags);

  const [status, setStatus] = useState<"ERROR" | "SUCCESS">("SUCCESS");
  const [stickyData, setStickyData] = useState<StickyThumb[]>([]);
  const [errorMessage, setErrorMessage] = useState<string>("");

  const emitSearchCounter = useRef(0);
  const receivedSearchCounter = useRef(0);
  useEffect(() => {
    emitSearchCounter.current++;
    const taskId = emitSearchCounter.current;
    let toSearchStatement = stmt;
    if (hideNSFW) {
      toSearchStatement += join(
        "",
        map((tag) => ` -${tag}`, nsfwTags)
      );
    }
    searchSticky(toSearchStatement, page)
      .then((sticky) => {
        if (sticky && taskId > receivedSearchCounter.current) {
          setStickyData(sticky);
          setStatus("SUCCESS");
        }
      })
      .catch((e) => {
        setErrorMessage(e);
        setStatus("ERROR");
      });
  }, [page, stmt, hideNSFW, nsfwTags]);

  if (status == "ERROR") {
    return (
      <InfoView
        className={props.className}
        title="Error"
        description={errorMessage}
        other={() => (
          <p className="italic text-muted-foreground pt-2">
            * This could happend when current search statement is not valid
          </p>
        )}
      />
    );
  }

  return <StickyGrid stickies={stickyData} page={page} {...props} />;
}
