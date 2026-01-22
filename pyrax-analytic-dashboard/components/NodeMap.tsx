"use client";

import React, { useMemo } from "react";
import {
    ComposableMap,
    Geographies,
    Geography,
    Marker,
    ZoomableGroup
} from "react-simple-maps";
import { NodeInfo } from "@/lib/types";

const geoUrl = "https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json";

interface NodeMapProps {
    nodes: NodeInfo[];
}

// Deterministic pseudo-random generator for demo coordinates based on string
const getPseudoCoordinates = (str: string): [number, number] => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        hash = ((hash << 5) - hash) + str.charCodeAt(i);
        hash |= 0;
    }

    // Map hash to reasonable lat/lon (excluding poles)
    // Longitude: -180 to 180
    const lon = (Math.abs(hash * 13) % 360) - 180;
    // Latitude: -60 to 70 (populated areas)
    const lat = (Math.abs(hash * 7) % 130) - 60;

    return [lon, lat];
};

export function NodeMap({ nodes }: NodeMapProps) {
    const [locations, setLocations] = React.useState<Record<string, [number, number]>>({});

    React.useEffect(() => {
        const fetchLocations = async () => {
            const newLocations: Record<string, [number, number]> = {};

            // Process nodes to find unique IPs
            for (const node of nodes) {
                // Extract IP from endpoint (http://1.2.3.4:8545 -> 1.2.3.4)
                let ip = node.endpoint.replace('http://', '').replace('https://', '').split(':')[0];
                // Skip domains for now unless resolving, stick to IPs
                if (ip.match(/^\d+\.\d+\.\d+\.\d+$/)) {
                    // Check if we already have it
                    if (locations[node.endpoint]) continue;

                    try {
                        // Rate limit friendly fetch
                        await new Promise(r => setTimeout(r, 500));
                        const res = await fetch(`https://ipapi.co/${ip}/json/`);
                        if (res.ok) {
                            const data = await res.json();
                            if (data.latitude && data.longitude) {
                                newLocations[node.endpoint] = [data.longitude, data.latitude];
                            }
                        }
                    } catch (e) {
                        console.error("GeoIP fetch failed for", ip);
                    }
                } else {
                    // For domains (rpc.pyrax.org), use fixed locations or resolve
                    // Fallback to pseudo for non-IPs or errors
                    newLocations[node.endpoint] = getPseudoCoordinates(node.endpoint);
                }
            }

            if (Object.keys(newLocations).length > 0) {
                setLocations(prev => ({ ...prev, ...newLocations }));
            }
        };

        fetchLocations();
    }, [nodes.length]); // Only re-run if node count changes

    const markers = useMemo(() => {
        return nodes.map(node => {
            // Use real location if available, else fallback
            const coordinates = locations[node.endpoint] || getPseudoCoordinates(node.endpoint);
            return {
                name: node.endpoint,
                coordinates,
                reachable: node.reachable
            };
        });
    }, [nodes, locations]);

    return (
        <div className="bg-card border border-border rounded-xl overflow-hidden relative" style={{ height: "400px" }}>
            <div className="absolute top-4 left-4 z-10 bg-card/80 backdrop-blur px-3 py-1.5 rounded-lg border border-border text-xs font-mono shadow-sm">
                Global Distribution
            </div>

            <ComposableMap projection="geoMercator" projectionConfig={{ scale: 100 }}>
                <ZoomableGroup center={[0, 0]} zoom={1} maxZoom={4} minZoom={0.5}>
                    <Geographies geography={geoUrl}>
                        {({ geographies }: { geographies: any[] }) =>
                            geographies.map((geo) => (
                                <Geography
                                    key={geo.rsmKey}
                                    geography={geo}
                                    fill="#3f3f46"
                                    stroke="#52525b"
                                    strokeWidth={0.5}
                                    style={{
                                        default: { outline: "none" },
                                        hover: { fill: "#52525b", outline: "none" },
                                        pressed: { outline: "none" },
                                    }}
                                />
                            ))
                        }
                    </Geographies>

                    {markers.map(({ name, coordinates, reachable }) => (
                        <Marker key={name} coordinates={coordinates}>
                            <circle
                                r={4}
                                fill={reachable ? "#22c55e" : "#ef4444"}
                                stroke="#0a0a0f"
                                strokeWidth={1}
                                className="animate-pulse"
                            />
                            <circle
                                r={8}
                                fill={reachable ? "#22c55e" : "#ef4444"}
                                opacity={0.3}
                                className="animate-ping"
                            />
                        </Marker>
                    ))}
                </ZoomableGroup>
            </ComposableMap>

            <div className="absolute bottom-4 right-4 text-[10px] text-muted-foreground bg-card/50 px-2 py-1 rounded">
                Scroll to zoom • Drag to pan
            </div>
        </div>
    );
}
