function QueueSkeleton() {
  return (
    <div className="flex-1 flex flex-col w-full">
      <div className="animate-pulse space-y-4 w-full">
        <div className="bg-white/10 rounded-xl h-32 w-full" />
        {[...Array(3)].map((_, i) => (
          <div key={i} className="bg-white/5 rounded-xl h-16 w-full" />
        ))}
      </div>
    </div>
  );
}

export default QueueSkeleton;
