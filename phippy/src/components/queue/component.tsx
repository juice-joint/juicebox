import {
  closestCenter,
  DndContext,
  DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { AnimatePresence, motion } from "framer-motion";
import { useMemo, useState } from "react";
import { useReposition } from "../../api/mutations/useReposition";
import { useAuth } from "../../api/queries/useAuth";
import { useCurrentSong } from "../../api/queries/useCurrentSong";
import { useQueue } from "../../api/queries/useQueue";
import NowPlaying from "../now-playing/component";
import QueueSkeleton from "../queue-skeleton/component";
import { SearchDialog } from "../search-dialog/component";
import SongItem from "../song-item/component";

export const Queue = () => {
  const { mutate: reposition } = useReposition();
  const { data: queue, isLoading: queueLoading } = useQueue();
  const currentSong = useCurrentSong();
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const { isAuthenticated } = useAuth();
  const nextSongs = useMemo(() => {
    return queue?.slice(1, 100) ?? [];
  }, [queue]);
  const sensors = useSensors(
    useSensor(PointerSensor, {}),
    useSensor(TouchSensor, {}),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    const newIndex = over?.data.current?.sortable.index;
    const uuid = active.id.toString();

    if (active.id !== over?.id && isAuthenticated) {
      reposition({ position: newIndex + 1, song_uuid: uuid });
    }
  };

  return (
    <div className="flex -mt-16 pt-16 flex-1 flex-col min-h-screen bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 text-white">
      <div
        className={`flex-1 w-full mx-auto px-4 sm:px-8 py-6 sm:py-12 max-w-xl ${
          nextSongs.length > 0 ? "pb-24 sm:pb-32" : "pb-0 sm:pb-0"
        }`}
      >
        {queueLoading ? (
          <QueueSkeleton />
        ) : (
          <>
            {!!currentSong && currentSong.name && (
              <NowPlaying currentSong={currentSong} />
            )}

            <div className="w-full space-y-3">
              {nextSongs.length > 0 ? (
                <DndContext
                  sensors={sensors}
                  collisionDetection={closestCenter}
                  onDragEnd={handleDragEnd}
                  modifiers={[restrictToVerticalAxis]}
                >
                  <SortableContext
                    items={nextSongs.map((song) => song.uuid)}
                    strategy={verticalListSortingStrategy}
                  >
                    <AnimatePresence initial={false}>
                      {nextSongs.map((song, i) => (
                        <motion.div
                          key={song.uuid}
                          layout
                          initial={{ opacity: 0, y: 20 }}
                          animate={{ opacity: 1, y: 0 }}
                          exit={{ opacity: 0, y: -20 }}
                          transition={{ duration: 0.2, ease: "easeInOut" }}
                        >
                          <SongItem key={song.uuid} i={i} song={song} />
                        </motion.div>
                      ))}
                    </AnimatePresence>
                  </SortableContext>
                </DndContext>
              ) : (
                <button
                  className="p-4 bg-white/5 backdrop-blur-sm w-full rounded-xl text-white"
                  onClick={() => setIsSearchOpen(true)}
                >
                  No Songs in Queue
                </button>
              )}
            </div>
          </>
        )}
      </div>

      <div className="fixed bottom-0 left-0 right-0 p-4">
        <div className="max-w-xl mx-auto">
          <button
            onClick={() => setIsSearchOpen(true)}
            className="w-full px-4 py-4 bg-white rounded-lg transition-colors text-black font-medium"
          >
            Search for Songs
          </button>
        </div>
      </div>

      <SearchDialog
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
      />
    </div>
  );
};
