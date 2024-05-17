import clsx from "clsx";
import { Link, LinkProps } from "react-router-dom";

interface NavProps extends React.HTMLAttributes<HTMLElement> {}
export function Nav({ className, ...props }: NavProps) {
  return <aside className={clsx("pb-12 border-r", className)} {...props}></aside>;
}

interface NavCatalogProps extends React.HTMLAttributes<HTMLDivElement> {}
export function NavCatalog({ className, ...props }: NavCatalogProps) {
  return <div className={clsx("space-y-4 py-4", className)} {...props}></div>;
}

interface NavCatalogTitleProps
  extends React.HTMLAttributes<HTMLHeadingElement> {}
export function NavCatalogTitle({ className, ...props }: NavCatalogTitleProps) {
  return (
    <h2
      className={clsx(
        "mb-2 px-4 text-lg font-semibold tracking-tight",
        className
      )}
      {...props}
    ></h2>
  );
}

interface NavCatalogContentListProps
  extends React.HTMLAttributes<HTMLUListElement> {}
export function NavCatalogContentList({
  className,
  ...props
}: NavCatalogContentListProps) {
  return <ul className={clsx("space-y-1", className)} {...props}></ul>;
}

interface NavCatalogContentListItemProps
  extends React.RefAttributes<HTMLAnchorElement>,
    LinkProps {
  isSelected?: boolean;
}
export function NavCatalogContentListItem({
  className,
  isSelected,
  ...props
}: NavCatalogContentListItemProps) {
  return (
    <li>
      <Link
        className={clsx(
          "inline-flex items-center whitespace-nowrap rounded-md text-sm font-medium transition-colors",
          "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
          "disabled:pointer-events-none disabled:opacity-50",
          "h-9 px-4 py-2 w-full justify-start",
          isSelected
            ? "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80"
            : "hover:bg-accent hover:text-accent-foreground"
        )}
        {...props}
      ></Link>
    </li>
  );
}
