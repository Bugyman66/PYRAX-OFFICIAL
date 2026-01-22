import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { AppProviders } from "@/components/AppProviders";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
    title: "PYRAX Analytic Dashboard",
    description: "Network monitor for PYRAX blockchain",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en" className="dark">
            <body className={inter.className}>
                <AppProviders>
                    {children}
                </AppProviders>
            </body>
        </html>
    );
}
