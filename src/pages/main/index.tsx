import { Outlet, useLocation } from "react-router-dom";
import {
  LuBarChart,
  LuCloud,
  LuHeart,
  LuHistory,
  LuLayoutGrid,
  LuNetwork,
  LuPackage,
  LuPlug,
  LuSettings,
} from "react-icons/lu";
import {
  Nav,
  NavCatalog,
  NavCatalogContentList,
  NavCatalogContentListItem,
  NavCatalogTitle,
} from "@/components/ui/nav";

export default function MainPage() {
  const location = useLocation();

  return (
    <div className="h-screen w-screen bg-background grid grid-cols-5">
      <Nav className="h-screen">
        <NavCatalog>
          <NavCatalogTitle>Library</NavCatalogTitle>
          <NavCatalogContentList>
            <NavCatalogContentListItem
              to="/"
              isSelected={location.pathname == "/"}
            >
              <LuLayoutGrid className="mr-2 h-4 w-4" />
              All
            </NavCatalogContentListItem>

            <NavCatalogContentListItem
              to="/fav"
              isSelected={location.pathname == "/fav"}
            >
              <LuHeart className="mr-2 h-4 w-4" />
              Favourite
            </NavCatalogContentListItem>

            <NavCatalogContentListItem
              to="/packages"
              isSelected={location.pathname == "/packages"}
            >
              <LuPackage className="mr-2 h-4 w-4" />
              Packages
            </NavCatalogContentListItem>

            <NavCatalogContentListItem
              to="/subscription"
              isSelected={location.pathname == "/subscription"}
            >
              <LuCloud className="mr-2 h-4 w-4" />
              Subscription
            </NavCatalogContentListItem>
            <NavCatalogContentListItem
              to="/history"
              isSelected={location.pathname == "/history"}
            >
              <LuHistory className="mr-2 h-4 w-4" />
              History
            </NavCatalogContentListItem>
          </NavCatalogContentList>
        </NavCatalog>
        <NavCatalog>
          <NavCatalogTitle>Apps</NavCatalogTitle>
          <NavCatalogContentList>
            <NavCatalogContentListItem
              to="/sync"
              isSelected={location.pathname == "/sync"}
            >
              <LuNetwork className="mr-2 h-4 w-4" />
              Sync
            </NavCatalogContentListItem>
            <NavCatalogContentListItem
              to="/plugins"
              isSelected={location.pathname == "/plugins"}
            >
              <LuPlug className="mr-2 h-4 w-4" />
              Plugins
            </NavCatalogContentListItem>
            <NavCatalogContentListItem
              to="/data"
              isSelected={location.pathname == "/data"}
            >
              <LuBarChart className="mr-2 h-4 w-4" />
              Database
            </NavCatalogContentListItem>
            <NavCatalogContentListItem
              to="/settings"
              isSelected={location.pathname == "/settings"}
            >
              <LuSettings className="mr-2 h-4 w-4" />
              Settings
            </NavCatalogContentListItem>
          </NavCatalogContentList>
        </NavCatalog>
      </Nav>
      <div className="col-span-4">
        <Outlet />
      </div>
    </div>
  );
}
