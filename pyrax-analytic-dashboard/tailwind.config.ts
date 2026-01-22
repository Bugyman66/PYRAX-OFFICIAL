import type { Config } from "tailwindcss";

const config: Config = {
    content: [
        "./app/**/*.{js,ts,jsx,tsx,mdx}",
        "./components/**/*.{js,ts,jsx,tsx,mdx}",
    ],
    theme: {
        extend: {
            colors: {
                background: "#0a0a0f",
                card: "#1a1a24",
                border: "#27272a",
                primary: {
                    DEFAULT: "#6366f1",
                    foreground: "#ffffff",
                },
                secondary: {
                    DEFAULT: "#12121a",
                    foreground: "#a1a1aa",
                },
                muted: {
                    DEFAULT: "#27272a",
                    foreground: "#71717a",
                },
                accent: {
                    DEFAULT: "#6366f1",
                    foreground: "#ffffff",
                },
                success: "#22c55e",
                warning: "#eab308",
                danger: "#ef4444",
            },
        },
    },
    plugins: [],
};
export default config;
