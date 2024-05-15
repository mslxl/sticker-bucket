import { Outlet } from "react-router-dom";
import { LuBarChart, LuCloud, LuFolder, LuHistory, LuNetwork, LuPackage, LuPlug, LuSettings } from "react-icons/lu";
import {
  Nav,
  NavCatalog,
  NavCatalogContentList,
  NavCatalogContentListItem,
  NavCatalogTitle,
} from "@/components/ui/nav";

export default function MainPage() {
  return (
    <div className="h-screen w-screen bg-background grid gird-cols-5">
      <Nav className="h-screen max-w-56">
        <NavCatalog>
          <NavCatalogTitle>Library</NavCatalogTitle>
          <NavCatalogContentList>
            <NavCatalogContentListItem to="/">
              <LuFolder className="mr-2 h-4 w-4" />
              All
            </NavCatalogContentListItem>

            <NavCatalogContentListItem to="/packages">
              <LuPackage className="mr-2 h-4 w-4" />
              Packages
            </NavCatalogContentListItem>

            <NavCatalogContentListItem to="/subscription">
              <LuCloud className="mr-2 h-4 w-4" />
              Subscription
            </NavCatalogContentListItem>
            <NavCatalogContentListItem to="/history">
              <LuHistory className="mr-2 h-4 w-4" />
              History
            </NavCatalogContentListItem>
          </NavCatalogContentList>
        </NavCatalog>
        <NavCatalog>
          <NavCatalogTitle>Apps</NavCatalogTitle>
          <NavCatalogContentList>
            <NavCatalogContentListItem to="/sync">
              <LuNetwork className="mr-2 h-4 w-4" />
              Sync
            </NavCatalogContentListItem>
            <NavCatalogContentListItem to="/plugin">
              <LuPlug className="mr-2 h-4 w-4" />
              Plugins
            </NavCatalogContentListItem>
            <NavCatalogContentListItem to="/data">
              <LuBarChart className="mr-2 h-4 w-4" />
              Database
            </NavCatalogContentListItem>
            <NavCatalogContentListItem to="/settings">
              <LuSettings className="mr-2 h-4 w-4" />
              Settings
            </NavCatalogContentListItem>
          </NavCatalogContentList>
        </NavCatalog>
      </Nav>
      <div className="col-span-4">
        <Outlet/>
      </div>
    </div>
  );
}
