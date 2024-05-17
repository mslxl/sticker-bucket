import * as dialog from "@tauri-apps/plugin-dialog";
import { exit } from "@tauri-apps/plugin-process";
export async function exitWithError(
  code: number,
  message: string | Error
): Promise<never> {
  if (typeof message == "string") {
    await dialog.message(message, {
      title: "Fatal Error",
      kind: "error",
    });
    await exit(code);
    return code as never;
  } else if (message instanceof Error) {
    return exitWithError(
      code,
      `[${message.name}]: ${message.message}\n${message.stack}`.trim()
    );
  } else {
    return exitWithError(code, "Unknown Error");
  }
}


export async function unreachable(){
    return exitWithError(-2, "Executated unreachable code")
}