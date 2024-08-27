import Logo from "@/../src-tauri/icons/32x32.png";
import {
  Nav,
  NavCatalog,
  NavCatalogContentList,
  NavCatalogContentListItem,
} from "@/components/nav";
import { Suspense } from "react";
import { LuBadge, LuLibrary, LuNetwork, LuSettings } from "react-icons/lu";
import { Outlet, useLoaderData } from "react-router-dom";
import { AppInfo } from "./index-loader";
export default function StartupPage() {
  const appInfo = useLoaderData() as AppInfo;

  return (
    <div className="h-screen w-screen flex items-stretch">
      <div className="h-full">
        <div className="h-full flex flex-col">
          <div className="my-4 p-4 flex items-center space-x-4">
            <img src={Logo} />
            <div>
              <h3 className="font-semibold leading-none tracking-tight">
                {appInfo.appName}
              </h3>
              <p className="text-sm text-muted-foreground">
                v{appInfo.appVersion}
              </p>
            </div>
          </div>
          <Nav border className="px-4 flex-1">
            <NavCatalog>
              <NavCatalogContentList>
                <NavCatalogContentListItem
                  isSelected
                  to="/"
                  className="hover:bg-background"
                >
                  <LuLibrary className="mr-2 h-4 w-4" />
                  Libraries
                </NavCatalogContentListItem>
                <NavCatalogContentListItem to="/">
                  <LuNetwork className="mr-2 h-4 w-4" />
                  Remote
                </NavCatalogContentListItem>
                <NavCatalogContentListItem to="/">
                  <LuSettings className="mr-2 h-4 w-4" />
                  Customize
                </NavCatalogContentListItem>
                <NavCatalogContentListItem to="/">
                  <LuBadge className="mr-2 h-4 w-4" />
                  About
                </NavCatalogContentListItem>
              </NavCatalogContentList>
            </NavCatalog>
          </Nav>
        </div>
      </div>
      <div className="flex-1 pt-[32px]">
        <Suspense>
          <Outlet />
        </Suspense>
      </div>
    </div>
  );
}
