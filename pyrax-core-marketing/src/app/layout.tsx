import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "PYRAX Proofing Hub",
  description: "Brand compliance and content review platform for PYRAX Blockchain",
  icons: {
    icon: "/favicon.ico",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-stone-950 text-stone-50 antialiased">
        {children}
      </body>
    </html>
  );
}
