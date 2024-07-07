import PrefSwitch from "@/components/pref/pref-switch";
import { useAtom } from "jotai";
import cfg from "@/store/settings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import PrefList from "@/components/pref/pref-list";
import { ZOD_TAG } from "@/const";

export default function DisplaySettingSection() {
  const [darkMode, setDarkMode] = useAtom(cfg.display.darkMode)
  const [blurNsfw, setBlurNsfw] = useAtom(cfg.display.blurNsfw);
  const [hideNsfw, setHideNsfw] = useAtom(cfg.display.hideNsfw);
  const [nsfwTags, setNsfwTags] = useAtom(cfg.display.nsfwTags);

  return (
    <ul className="space-y-4">
      <li>
        <Card>
          <CardHeader>
            <CardTitle>UI</CardTitle>
          </CardHeader>
          <CardContent>
            <ul>
              <li className="py-2">
                <PrefSwitch
                  leading="Dark Mode"
                  description="Use dark colorplatte"
                  checked={darkMode}
                  onCheckedChange={setDarkMode}
                />
              </li>
            </ul>
          </CardContent>
        </Card>
      </li>
      <li>
        <Card>
          <CardHeader>
            <CardTitle>Hide Tag</CardTitle>
          </CardHeader>
          <CardContent>
            <ul>
              <li className="py-2">
                <PrefSwitch
                  leading="Hide NSFW"
                  description="Never display sticker with specify tag"
                  disabled={blurNsfw}
                  checked={hideNsfw}
                  onCheckedChange={setHideNsfw}
                />
              </li>
              <li className="border-t py-2">
                <PrefSwitch
                  leading="Blur NSFW"
                  description="Blur sticker with specify tag"
                  disabled={hideNsfw}
                  checked={blurNsfw}
                  onCheckedChange={setBlurNsfw}
                />
              </li>
              <li className="border-t py-2">
                <PrefList
                  leading="NSFW Tag"
                  description="The sticker that has specify tag to be hidden or blur"
                  items={nsfwTags}
                  onItemsChanged={setNsfwTags}
                  itemZod={ZOD_TAG}
                  itemDescription="Input must be the format of 'namespace:value'"
                />
              </li>
            </ul>
          </CardContent>
        </Card>
      </li>
    </ul>
  );
}
