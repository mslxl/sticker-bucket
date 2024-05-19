import { StickyThumb } from "@/lib/cmd/library";
import { StickyListProps } from ".";
import { convertFileSrc } from "@tauri-apps/api/core";

interface GridItemProps {
  sticky: StickyThumb;
}

function GridItem({ sticky }: GridItemProps) {
  const url = convertFileSrc(sticky.path);
  return (
    <li>
      <img src={url} />
      <span>{sticky.name}</span>
    </li>
  );
}

export default function StickyGrid({ stickies }: StickyListProps) {
  return (
    <ul>
      {stickies.map((s) => (
        <GridItem key={s.path} sticky={s} />
      ))}
    </ul>
  );
}
