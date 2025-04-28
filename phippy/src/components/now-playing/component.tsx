import { FormattedSong } from "../../api/api-types";

function NowPlaying({ currentSong }: { currentSong: FormattedSong }) {
  return (
    <div className="w-full mb-6 p-6 sm:p-8 bg-black/30 backdrop-blur-lg rounded-2xl border border-white/10 shadow-xl">
      <div className="flex items-center gap-2 mb-4">
        <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
        <h2 className="text-sm uppercase tracking-wider text-white/70 font-medium">
          Now Playing
        </h2>
      </div>
      <div className="flex items-center gap-4">
        <div className="flex-1">
          <p className="text-xl sm:text-2xl font-bold text-white mb-1">
            {currentSong?.formattedName}
          </p>
        </div>
        <div className="flex items-center">
          {currentSong.status === "InProgress" && (
            <div className="animate-spin h-4 w-4 border-2 border-purple-200/90 rounded-full border-t-transparent" />
          )}
          {currentSong.status === "Failed" && (
            <div className="text-red-400 flex items-center gap-2 bg-red-500/10 px-3 py-1.5 rounded-full">
              <svg
                className="w-5 h-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
              <span className="text-sm font-medium">Failed</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default NowPlaying;
