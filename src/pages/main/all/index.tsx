import StickyList from "@/components/sticky-list";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { popModalAtom, pushDialogAtom } from "@/store/modal";
import { useSetAtom } from "jotai";
import { LucideListCollapse } from "lucide-react";
import { lazy, useState } from "react";
import {
  LuFolderPlus,
  LuImagePlus,
  LuMenu,
  LuSearch,
  LuSearchSlash,
  LuTextCursorInput,
} from "react-icons/lu";
import { useNavigate } from "react-router-dom";

export default function AllPage() {
  const pushDialog = useSetAtom(pushDialogAtom);
  const popModal = useSetAtom(popModalAtom);
  const navigate = useNavigate()
  function showStickyAddDialog() {
    pushDialog(lazy(() => import("@/components/sticky-add-file"))).finally(
      () => {
        popModal();
      }
    );
  }
  function showTextAddDialog() {
    pushDialog(lazy(() => import("@/components/text-add"))).finally(() => {
      popModal();
    });
  }
  function addStickyFromFolder(){
    navigate('add/folder')

  }

  const [searchInput, setSearchInput] = useState("");
  const [page, setPage] = useState(0);

  return (
    <div className="h-screen">
      <nav className="flex items-center p-2 h-16 border-b bg-background">
        <div className="relative ml-auto flex-1 md:grow-0">
          <LuSearch className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            type="search"
            placeholder="Search..."
            className="w-full rounded-lg bg-background pl-8 md:w-[200px] lg:w-[336px]"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
          />
        </div>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button>
              <LuMenu />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>
              <LuSearchSlash className="mr-2 h-4 w-4" />
              <span>Advanced Search</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem onClick={showStickyAddDialog}>
                <LuImagePlus className="mr-2 h-4 w-4" />
                <span>Add Sticky</span>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={showTextAddDialog}>
                <LuTextCursorInput className="mr-2 h-4 w-4" />
                <span>Add Text</span>
              </DropdownMenuItem>
              <DropdownMenuSub>
                <DropdownMenuSubTrigger>
                  <LucideListCollapse className="mr-2 h-4 w-4" />
                  <span>More</span>
                </DropdownMenuSubTrigger>
                <DropdownMenuPortal>
                  <DropdownMenuSubContent>
                    <DropdownMenuItem onClick={addStickyFromFolder}>
                      <LuFolderPlus className="mr-2 h-4 w-4" />
                      <span>Add Stickies From Folder</span>
                    </DropdownMenuItem>
                  </DropdownMenuSubContent>
                </DropdownMenuPortal>
              </DropdownMenuSub>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </nav>
      <StickyList
        className="h-[calc(100%-4rem)] overflow-y-auto p-4"
        stmt={searchInput}
        page={page}
        onPageChange={setPage}
      />
    </div>
  );
}
