import { useState } from "react";
import { useEventSource } from "./api/sse/useEventSource";
import { AdminDialog } from "./components/admin-dialog/component";
import { ErrorScreen } from "./components/error/component";
import { Header } from "./components/header/component";
import { Queue } from "./components/queue/component";

export const App = () => {
  const { error } = useEventSource();
  const [isAdminOpen, setIsAdminOpen] = useState(false);

  if (error) {
    return <ErrorScreen />;
  }

  return (
    <div className="flex flex-col w-full h-[100dvh] max-h-[100dvh] overflow-hidden bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900">
      <Header onOpenAdmin={() => setIsAdminOpen(!isAdminOpen)} />
      <AdminDialog className={`admin-dialog ${isAdminOpen ? "open" : ""}`} />
      <div className="flex-1 min-h-0 overflow-auto">
        <Queue />
      </div>
    </div>
  );
};

export default App;
