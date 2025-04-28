import { useState } from "react";
import { useDebouncedCallback } from "use-debounce";
import { useAddToQueue } from "../../api/mutations/useAddToQueue";
import { useSearchYoutube } from "../../api/mutations/useSearchYoutube";

export const SearchDialog = ({
  isOpen,
  onClose,
}: {
  isOpen: boolean;
  onClose: () => void;
}) => {
  const [displayQuery, setDisplayQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const {
    data: results = [],
    isPending,
    isError,
    error,
  } = useSearchYoutube(debouncedQuery);

  const { mutate: addToQueue } = useAddToQueue();
  const [shouldPitchShift, setShouldPitchShift] = useState(false);

  const debouncedSetQuery = useDebouncedCallback((value: string) => {
    setDebouncedQuery(value);
  }, 300);

  if (!isOpen) return null;

  function SearchSkeleton() {
    return (
      <div className="animate-pulse space-y-4 w-full">
        {[...Array(4)].map((_, i) => (
          <div
            key={i}
            className="p-4 bg-gray-800/50 backdrop-blur-sm rounded-xl flex gap-4 items-center"
          >
            <div className="bg-white/10 rounded-lg w-20 h-20" />
            <div className="space-y-2 flex-1">
              <div className="bg-white/10 rounded h-4 w-3/4" />
              <div className="bg-white/10 rounded h-4 w-1/2" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div
      onClick={(e) => {
        e.stopPropagation();
        onClose();
      }}
      className="z-50 fixed inset-0 bg-gradient-to-br from-purple-900/80 via-indigo-900/80 to-blue-900/80 backdrop-blur-sm flex items-center justify-center p-8"
    >
      <div
        onClick={(e) => {
          e.stopPropagation();
        }}
        className="bg-gray-900/90 backdrop-blur-sm rounded-2xl w-full max-w-2xl border border-white/20"
      >
        <div className="flex justify-between items-center p-6 border-b border-white/10">
          <h2 className="text-2xl font-medium text-white">Search Songs</h2>
          <div className="flex items-center gap-2">
            <label className="relative flex items-center cursor-pointer">
              <input
                type="checkbox"
                id="pitch-shift"
                checked={shouldPitchShift}
                onChange={(e) => setShouldPitchShift(e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600"></div>
              <span className="ml-2 text-sm font-medium text-white/80">
                Multi-Key
              </span>
            </label>
          </div>
          <button
            onClick={onClose}
            className="text-white/60 hover:text-white/90 transition-colors"
          >
            x
          </button>
        </div>

        <div className="p-6 space-y-6">
          <div className="relative">
            <input
              placeholder="search for a song"
              autoComplete="off"
              autoCorrect="off"
              spellCheck="false"
              type="text"
              value={displayQuery}
              onChange={(e) => {
                setDisplayQuery(e.target.value);
                debouncedSetQuery(e.target.value);
              }}
              className={`w-full px-4 py-3 bg-gray-800/90 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/50 ${
                isPending && debouncedQuery ? "animate-spin-loading" : ""
              } focus:border-transparent text-white placeholder-white/40`}
            />
            <button
              className={`absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded ${
                displayQuery.trim()
                  ? "text-purple-300 hover:text-purple-200"
                  : "text-white/30"
              }`}
              disabled={!displayQuery.trim()}
            />
          </div>

          <div className="max-h-[60vh] overflow-y-auto">
            {isPending && debouncedQuery && <SearchSkeleton />}

            {isError && (
              <div className="flex justify-center items-center py-8 text-red-400">
                {error?.message || "something went wrong fr"}
              </div>
            )}

            {!isPending &&
              !isError &&
              results?.length === 0 &&
              debouncedQuery && (
                <div className="flex justify-center items-center py-8 text-white/60">
                  no results found
                </div>
              )}

            {!isError &&
              results?.map((result) => (
                <div
                  key={result.url}
                  className="p-4 bg-gray-800/80 backdrop-blur-sm rounded-xl hover:bg-gray-700/80 transition-colors cursor-pointer flex gap-4 items-center mb-3"
                  onClick={() => {
                    addToQueue({
                      yt_link: result.url,
                      name: result.title
                        .replace(/[^a-zA-Z0-9\s]/g, "")
                        .replace(/\s/g, "_"),
                      is_key_changeable: shouldPitchShift,
                    });
                    onClose();
                  }}
                >
                  <img
                    src={`https://img.youtube.com/vi/${result.id}/0.jpg`}
                    alt={result.title}
                    className="w-20 h-20 object-cover rounded-lg"
                  />
                  <div>
                    <p className="font-medium text-white">{result.title}</p>
                    <p className="text-sm text-white/60">{result.title}</p>
                  </div>
                </div>
              ))}
          </div>
        </div>
      </div>
    </div>
  );
};
