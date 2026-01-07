// Root layout - delegates to locale layout for html/body rendering
// This prevents hydration errors from nested html/body tags

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return children;
}
