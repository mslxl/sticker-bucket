import InfoView from "@/components/info-view";
import { Button } from "@/components/ui/button";

interface BatchAddCompleted {
  total: number;
}
export default function BatchAddCompleted({ total }: BatchAddCompleted) {
  return (
    <InfoView
    className="h-screen"
      title="Completed"
      description={`Total add ${total} stickies`}
      other={() => (
        <div className="p-4">
          <Button onClick={() => history.back()}>Back</Button>
        </div>
      )}
    />
  );
}
