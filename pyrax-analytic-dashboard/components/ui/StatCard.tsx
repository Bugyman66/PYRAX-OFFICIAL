import { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface StatCardProps {
    label: string;
    value: string | number;
    icon?: LucideIcon;
    trend?: string;
    status?: "default" | "success" | "warning" | "danger";
    className?: string;
}

export function StatCard({
    label,
    value,
    icon: Icon,
    trend,
    status = "default",
    className
}: StatCardProps) {
    const statusColors = {
        default: "text-white",
        success: "text-success",
        warning: "text-warning",
        danger: "text-danger",
    };

    return (
        <div className={cn(
            "bg-card border border-border rounded-xl p-6 transition-all hover:border-primary/50",
            className
        )}>
            <div className="flex items-center justify-between mb-4">
                <span className="text-secondary-foreground text-xs uppercase tracking-wider font-medium">
                    {label}
                </span>
                {Icon && <Icon className="w-5 h-5 text-muted-foreground" />}
            </div>

            <div className="flex items-baseline gap-2">
                <span className={cn(
                    "text-3xl font-bold tabular-nums tracking-tight",
                    statusColors[status]
                )}>
                    {value}
                </span>
                {trend && (
                    <span className="text-xs text-muted-foreground">
                        {trend}
                    </span>
                )}
            </div>
        </div>
    );
}
