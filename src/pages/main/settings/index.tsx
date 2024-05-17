import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import DisplaySettingSection from "./display";
import DatabaseSettingSection from "./database";

export default function SettingsPage() {
  return (
    <div className="p-4">
      <Tabs defaultValue="display">
        <TabsList>
          <TabsTrigger value="display">Display</TabsTrigger>
          <TabsTrigger value="database">Database</TabsTrigger>
        </TabsList>
        <TabsContent value="display">
          <DisplaySettingSection />
        </TabsContent>
        <TabsContent value="database">
            <DatabaseSettingSection/>
        </TabsContent>
      </Tabs>
    </div>
  );
}
