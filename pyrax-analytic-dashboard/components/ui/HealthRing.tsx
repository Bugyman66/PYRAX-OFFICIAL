"use client";

import { cn } from "@/lib/utils";

interface HealthRingProps {
    percentage: number;
    size?: number;
    strokeWidth?: number;
    label?: string;
    subLabel?: string;
    className?: string;
}

export function HealthRing({
    percentage,
    size = 120,
    strokeWidth = 10,
    label,
    subLabel,
    className
}: HealthRingProps) {
    const radius = (size - strokeWidth) / 2;
    const circumference = radius * 2 * Math.PI;
    const offset = circumference - (percentage / 100) * circumference;

    let colorClass = "text-success";
    if (percentage < 70) colorClass = "text-warning";
    if (percentage < 30) colorClass = "text-danger";

    return (
        <div className={cn("relative flex flex-col items-center", className)}>
            <svg
                width={size}
                height={size}
                viewBox={`0 0 ${size} ${size}`}
                className="transform -rotate-90"
            >
                {/* Background Ring */}
                <circle
                    cx={size / 2}
                    cy={size / 2}
                    r={radius}
                    fill="transparent"
                    stroke="currentColor"
                    strokeWidth={strokeWidth}
                    className="text-secondary"
                />
                {/* Progress Ring */}
                <circle
                    cx={size / 2}
                    cy={size / 2}
                    r={radius}
                    fill="transparent"
                    stroke="currentColor"
                    strokeWidth={strokeWidth}
                    strokeDasharray={circumference}
                    strokeDashoffset={offset}
                    strokeLinecap="round"
                    className={cn("transition-all duration-1000 ease-out", colorClass)}
                />
            </svg>

            <div className="absolute inset-0 flex flex-col items-center justify-center text-center">
                <span className={cn("text-2xl font-bold tabular-nums", colorClass)}>
                    {Math.round(percentage)}%
                </span>
                {label && (
                    <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
                        {label}
                    </span>
                )}
            </div>

            {subLabel && (
                <p className="mt-2 text-sm text-muted-foreground">{subLabel}</p>
            )}
        </div>
    );
}
