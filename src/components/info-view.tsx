import clsx from "clsx";
import { FC, ReactNode } from "react";
import { LuAlertTriangle } from "react-icons/lu";

export interface ErrorViewProps {
  className?: string;
  icon?: ReactNode;
  title?: string;
  description: string;

  other?: FC<{}>;
}
export default function InfoView({
  className,
  icon = <LuAlertTriangle className="mr-2" />,
  title = "Error",
  description,
  other: OtherComponent = () => <></>,
}: ErrorViewProps) {
  return (
    <div
      className={clsx("flex justify-center items-center flex-col", className)}
    >
      <h4 className="flex items-center justify-center font-semibold p-2 text-lg leading-none">
        {icon}
        {title}
      </h4>
      <p>{description}</p>
      <OtherComponent />
    </div>
  );
}
