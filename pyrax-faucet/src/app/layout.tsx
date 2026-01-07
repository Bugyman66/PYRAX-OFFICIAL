import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

export const metadata: Metadata = {
  title: "PYRAX Faucet - Get Testnet Tokens",
  description: "Get free PYRAX testnet tokens for development and testing on the PYRAX Network.",
  icons: {
    icon: "/pyrax-favicon.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.variable} font-sans antialiased bg-stone-950 text-stone-50 min-h-screen`}>
        {children}
      </body>
    </html>
  );
}
