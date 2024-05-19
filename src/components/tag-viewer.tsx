import { FC, useMemo } from "react";
import { Badge } from "./ui/badge";
import {
  groupBy,
  map,
  concat,
  prop,
  reduce,
  sortBy,
  identical,
  curryN,
} from "lodash/fp";
import clsx from "clsx";

export interface TagNS {
  namespace: string;
}

export interface Tag {
  namespace: string;
  value: string;
}

export const tagEq = curryN(
  2,
  (lhs: Tag, rhs: Tag) => lhs.namespace == rhs.namespace && rhs.value == lhs.value
);
export const tagNe = curryN(2,
  (lhs: Tag, rhs: Tag) => !tagEq(lhs, rhs)
)

export interface TagGroup {
  namespace: string;
  values: string[];
}
export interface TagViewerProps {
  className?: string;
  tags: Tag[] | TagGroup[];
  valueRender?: FC<Tag>;
  namespaceRender?: FC<TagNS>;
}

function DefaultNamespaceRender({ namespace }: TagNS) {
  return <b>{namespace}</b>;
}
function DefaultTagValueRender({ value }: Tag) {
  return <i>{value}</i>;
}

export default function TagViewer({
  className,
  tags,
  namespaceRender: NamespaceRender = DefaultNamespaceRender,
  valueRender: ValueRender = DefaultTagValueRender,
}: TagViewerProps) {
  const groupedTag = useMemo(
    () =>
      sortBy(
        prop("namespace"),
        map(
          (entry: Tag[] | TagGroup[]): TagGroup => ({
            namespace: entry[0].namespace,
            values: sortBy(
              identical,
              reduce(
                concat,
                [],
                map((tag: Tag | TagGroup): string | string[] => {
                  return (prop("value", tag) ?? prop("values", tag)) as
                    | string
                    | string[];
                }, entry)
              ) as string[]
            ),
          }),
          groupBy((tag: Tag | TagGroup) => tag.namespace, tags)
        ) as unknown as TagGroup[]
      ),
    [tags]
  );

  return (
    <ul className={clsx("p-2 space-y-0.5", className)}>
      {groupedTag.map((ns) => (
        <li className="flex gap-2" key={ns.namespace}>
          <div>
            <Badge>
              <NamespaceRender namespace={ns.namespace} />
            </Badge>
          </div>
          <ul className="space-x-1">
            {ns.values.map((value) => (
              <li key={value} className="float-start">
                <Badge variant="outline">
                  <ValueRender namespace={ns.namespace} value={value} />
                </Badge>
              </li>
            ))}
          </ul>
        </li>
      ))}
    </ul>
  );
}
