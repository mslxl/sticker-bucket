import {
  StickerThumb,
  countSearchStickerPage,
  searchSticker,
} from "@/lib/cmd/library";
import StickerGrid from "./sticker-grid";
import { useEffect, useRef, useState } from "react";
import { useAtomValue } from "jotai";
import cfg from "@/store/settings";
import { filter, join, map, range } from "lodash/fp";
import InfoView from "../info-view";

import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";

export interface StickerListProps {
  stmt: string;
  className?: string;
  page?: number;
  onPageChange?: (page: number) => void;
}

export interface StickerListLayoutProps extends Omit<StickerListProps, "stmt"> {
  stickies: StickerThumb[];
}

export default function StickerList({
  stmt,
  page = 1,
  onPageChange,
  className,
  ...props
}: StickerListProps) {
  const hideNSFW = useAtomValue(cfg.display.hideNsfw);
  const nsfwTags = useAtomValue(cfg.display.nsfwTags);

  const [status, setStatus] = useState<"ERROR" | "SUCCESS">("SUCCESS");
  const [stickerData, setStickerData] = useState<StickerThumb[]>([]);
  const [totalPage, setTotalPage] = useState(0);
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

    Promise.all([
      countSearchStickerPage(toSearchStatement),
      searchSticker(toSearchStatement, page),
    ])
      .then(([pg, sticker]) => {
        if (sticker && taskId > receivedSearchCounter.current) {
          setStickerData(sticker);
          setTotalPage(pg);
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
        title="Error"
        className={className}
        description={errorMessage}
        other={() => (
          <p className="italic text-muted-foreground pt-2">
            * This could happend when current search statement is not valid
          </p>
        )}
      />
    );
  }

  return (
    <div className={className}>
      <StickerGrid stickies={stickerData} page={page} {...props} />
      <Pagination className="p-4">
        <PaginationContent>
          {page == 1 ? null : (
            <PaginationItem>
              <PaginationPrevious
                onClick={() => onPageChange && onPageChange(page - 1)}
              />
            </PaginationItem>
          )}
          {page < 5 ? null : (
            <PaginationItem>
              <PaginationEllipsis />
            </PaginationItem>
          )}
          {filter(
            (v) => v >= 1 && v <= totalPage,
            range(page - 2, page + 3)
          ).map((idx) => (
            <PaginationItem key={idx}>
              <PaginationLink
                onClick={() => onPageChange && onPageChange(idx)}
                isActive={idx == page}
              >
                {idx}
              </PaginationLink>
            </PaginationItem>
          ))}
          {totalPage - page < 5 ? null : (
            <PaginationItem>
              <PaginationEllipsis />
            </PaginationItem>
          )}
          {page >= totalPage ? null : (
            <PaginationItem>
              <PaginationNext
                onClick={() => onPageChange && onPageChange(page + 1)}
              />
            </PaginationItem>
          )}
        </PaginationContent>
      </Pagination>
    </div>
  );
}
