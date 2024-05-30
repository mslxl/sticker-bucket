import InfoView from "@/components/info-view";
import { Button } from "@/components/ui/button";
import { useNavigate, useRouteError } from "react-router-dom";
import * as proc from "@tauri-apps/plugin-process";
import { error } from "@tauri-apps/plugin-log";

function getErrorMessage(err: any): string {
  if (err instanceof Error) {
    return `${err.name}: ${err.message}`;
  } else if (typeof err == "string") {
    return err;
  } else {
    return err.toString();
  }
}

export default function ErrorPage() {
  const routerErr = useRouteError();
  console.error(routerErr);
  const err = getErrorMessage(routerErr);
  error(err);
  const nav = useNavigate();

  function relaunch() {
    proc.relaunch();
  }
  function recover() {
    nav("/");
  }

  return (
    <div className="w-screen h-screen flex items-center justify-center">
      <InfoView
        title="Unexpected Application Error!"
        className="space-y-4"
        description={err}
        other={() => (
          <span className="flex items-center space-x-2">
            <Button variant="outline" onClick={recover}>
              Try Recover
            </Button>
            <Button variant="destructive" onClick={relaunch}>
              Relaunch App
            </Button>
          </span>
        )}
      />
    </div>
  );
}
