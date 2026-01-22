import { AlertTriangle, GitFork, Clock } from "lucide-react";
import { cn } from "@/lib/utils";

interface AlertBannerProps {
    stalled: boolean;
    forkDetected: boolean;
    heightDelta: number;
}

export function AlertBanner({ stalled, forkDetected, heightDelta }: AlertBannerProps) {
    if (!stalled && !forkDetected && heightDelta < 2) return null;

    return (
        <div className="flex flex-col gap-2 mb-6">
            {stalled && (
                <div className="bg-danger/10 border border-danger/50 rounded-lg p-4 flex items-center gap-4 text-danger animate-pulse">
                    <Clock className="w-5 h-5" />
                    <div className="flex-1">
                        <h3 className="font-bold">Network Stalled</h3>
                        <p className="text-sm opacity-90">Block production has stopped. Investigate immediately.</p>
                    </div>
                </div>
            )}

            {forkDetected && (
                <div className="bg-warning/10 border border-warning/50 rounded-lg p-4 flex items-center gap-4 text-warning">
                    <GitFork className="w-5 h-5" />
                    <div className="flex-1">
                        <h3 className="font-bold">Chain Fork Detected</h3>
                        <p className="text-sm opacity-90">Nodes are reporting conflicting block hashes at the same height.</p>
                    </div>
                </div>
            )}

            {heightDelta > 2 && (
                <div className="bg-warning/10 border border-warning/20 rounded-lg p-3 flex items-center gap-3 text-warning">
                    <AlertTriangle className="w-4 h-4" />
                    <span className="text-sm font-medium">
                        Consensus Warning: Nodes are out of sync by {heightDelta} blocks.
                    </span>
                </div>
            )}
        </div>
    );
}
