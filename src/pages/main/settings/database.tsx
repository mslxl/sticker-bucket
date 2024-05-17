import PrefFile from "@/components/pref/pref-file";
import { useAtom } from "jotai";
import cfg from '@/store/settings'

export default function DatabaseSettingSection() {

    const [stickyDataFolder, setStickyDataFolder] = useAtom(cfg.database.stickyDir)


  return (
    <ul>
      <li className="border-t py-2">
        <PrefFile
          leading="Sticky Data Folder"
          value={stickyDataFolder}
          open
          option={{
            title: 'Open sticky data folder',
            directory: true,
            recursive: false
          }}
          onValueChange={setStickyDataFolder}
          description={() => <span>Where stickies and databse located in.<br/><em>*Effects after restart</em></span>}
        />
      </li>
    </ul>
  );
}
