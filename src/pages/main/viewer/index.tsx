import StickerList from "@/components/sticker-list";
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
import { lazy, useEffect, useState } from "react";
import {
  LuFolderPlus,
  LuImagePlus,
  LuMenu,
  LuSearch,
  LuSearchSlash,
  LuTextCursorInput,
} from "react-icons/lu";
import { useNavigate, useParams } from "react-router-dom";

export interface AllPageParam{
  stmt?: string
  page?: string
}

export default function AllPage() {
  
  const {
    stmt: additionStatement = "",
    page: pageStr = '1'
  } = useParams() as AllPageParam

  const navigate = useNavigate();
  const page = parseInt(pageStr)
  function setPage(pg: number){
    if(additionStatement.length > 0){
      navigate(`/list/${additionStatement}/${pg}`)
    }else{
      navigate(`/list/${pg}`)
    }
  }

  const pushDialog = useSetAtom(pushDialogAtom);
  const popModal = useSetAtom(popModalAtom);
  function showStickerAddDialog() {
    pushDialog(lazy(() => import("@/components/sticker-add-file"))).finally(
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
  function addStickerFromFolder() {
    navigate("add/folder");
  }

  const [searchInput, setSearchInput] = useState("");
  useEffect(()=>{
    if(page > 1) setPage(1)
  }, [searchInput])

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
              <DropdownMenuItem onClick={showStickerAddDialog}>
                <LuImagePlus className="mr-2 h-4 w-4" />
                <span>Add Sticker</span>
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
                    <DropdownMenuItem onClick={addStickerFromFolder}>
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
      <StickerList
        className="h-[calc(100%-4rem)] overflow-y-auto p-4"
        stmt={`${searchInput} ${additionStatement}`}
        page={page}
        onPageChange={setPage}
      />
    </div>
  );
}
