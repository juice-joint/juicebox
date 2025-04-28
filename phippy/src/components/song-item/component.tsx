import { FormattedSong, Status } from "../../api/api-types";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useAuth } from "../../api/queries/useAuth";
import { motion, PanInfo, useMotionValue, useTransform } from "framer-motion";
import { useDeleteSong } from "../../api/mutations/useDeleteSong";

export default function SongItem({
  song,
  i,
}: {
  song: FormattedSong;
  i: number;
}) {
  const { isAuthenticated } = useAuth();
  const { mutate: deleteSong } = useDeleteSong();

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: song.uuid, disabled: !isAuthenticated });

  const x = useMotionValue(0);
  const opacity = useTransform(x, [-100, 0], [1, 0]);
  const background = useTransform(x, [-100, 0], ["#ef4444", "#ef444400"]);

  const handleDragEnd = (_event: any, info: PanInfo) => {
    if (info.offset.x < -100 && isAuthenticated) {
      deleteSong({ song_uuid: song.uuid });
    }
  };

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div className="relative" ref={setNodeRef} style={style}>
      <motion.div
        style={{ background, opacity }}
        className="absolute inset-0 rounded-lg flex items-center justify-end pr-4"
      >
        <svg
          className="w-6 h-6 text-white"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
          />
        </svg>
      </motion.div>
      <motion.div
        style={{ x }}
        drag={isAuthenticated ? "x" : false}
        dragConstraints={{ left: 0, right: 0 }}
        dragDirectionLock
        onDragEnd={handleDragEnd}
        className={`relative p-3 sm:p-4 bg-black/20 backdrop-blur-sm rounded-lg border border-white/10 ${
          song.status === Status.InProgress ? "bg-gray-700/50" : ""
        } 
        ${isDragging ? "z-10 opacity-50" : ""}`}
      >
        <div className="flex items-center gap-3 pr-12">
          {song.status === "InProgress" && (
            <div className="animate-spin h-4 w-4 border-2 border-purple-200/90 rounded-full border-t-transparent" />
          )}
          <span className="text-sm text-purple-200/90">#{i + 1}</span>
          <div className="flex-1 min-w-0">
            <p
              className={`text-base sm:text-lg truncate text-white ${
                song.status === "InProgress" ? "text-white/50" : ""
              }`}
            >
              {song.formattedName}
            </p>
          </div>
          {song.status === "Failed" && (
            <div className="text-red-400 flex items-center gap-2">
              <svg
                className="w-4 h-4"
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
              <span className="text-sm">Download Failed</span>
            </div>
          )}
          {isAuthenticated && (
            <div
              {...attributes}
              {...listeners}
              className="absolute right-0 inset-y-0 w-12 flex items-center justify-center touch-none hover:bg-white/5 active:bg-white/10 transition-colors rounded-lg"
            >
              <svg
                className="w-4 h-4 text-purple-200/50"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  d="M4 6h16M4 12h16M4 18h16"
                  strokeWidth="2"
                  stroke="currentColor"
                  strokeLinecap="round"
                />
              </svg>
            </div>
          )}
        </div>
      </motion.div>
    </div>
  );
}
