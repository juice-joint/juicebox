import { useState, useEffect } from "react";
import { useCurrentSong } from "./api/queries/useCurrentSong";
import { useEventSource } from "./api/sse/useEventSource";
import { ErrorScreen } from "./components/error/component";
import QRCodeBanner from "./components/qr-code/component";
import { Queue } from "./components/queue/component";
import { Splash } from "./components/splash/component";
import { VideoPlayer } from "./components/video-player";

enum View {
  HOME = "home",
  LOADING = "loading"
}

function App() {
  const [currentView, setCurrentView] = useState<View>(View.HOME);

  // Read query params on mount
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const viewParam = params.get('view');
    const view = Object.values(View).includes(viewParam as View) ? (viewParam as View) : View.HOME;
    setCurrentView(view);
  }, []);

  // Render based on current view
  switch (currentView) {
    case View.LOADING:
      return <Loading />;
    default:
      return <Home />;
  }
}

function Home() {
  const currentSong = useCurrentSong();
  const { error } = useEventSource();

  if (error) {
    return <ErrorScreen />;
  }

  return (
    <div className="w-full h-full">
      {!currentSong?.name && <Splash />}
      {currentSong?.name && <VideoPlayer />}
      <QRCodeBanner />
      <Queue />
    </div>
  );
}

function Loading() {
  return (
    <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900">
      <div className="text-center">
        <div className="relative">
          {/* Spinning loader */}
          <div className="w-24 h-24 border-4 border-white/20 border-t-white rounded-full animate-spin mx-auto mb-6"></div>
        </div>
        
        <p className="text-white text-3xl font-medium">Loading...</p>
      </div>
    </div>
  );
}

export default App;