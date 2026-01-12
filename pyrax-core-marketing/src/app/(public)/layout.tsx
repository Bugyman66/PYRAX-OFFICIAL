import Link from 'next/link';

export default function PublicLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-stone-950 flex flex-col">
      {/* Header */}
      <header className="bg-stone-900 border-b border-stone-800 sticky top-0 z-50">
        <div className="max-w-6xl mx-auto px-6">
          <div className="flex items-center justify-between h-16">
            <Link href="/" className="flex items-center gap-3">
              <span className="text-2xl font-bold pyrax-gradient-text">PYRAX</span>
            </Link>
            <nav className="flex items-center gap-6">
              <Link 
                href="/media-kit" 
                className="text-stone-400 hover:text-stone-100 transition-colors text-sm font-medium"
              >
                Media Kit
              </Link>
              <Link 
                href="/public-library" 
                className="text-stone-400 hover:text-stone-100 transition-colors text-sm font-medium"
              >
                Public Library
              </Link>
              <Link
                href="/auth/login"
                className="px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white text-sm font-medium rounded-lg transition-colors"
              >
                Sign In
              </Link>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-stone-900 border-t border-stone-800">
        <div className="max-w-6xl mx-auto px-6 py-12">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div className="md:col-span-2">
              <span className="text-xl font-bold pyrax-gradient-text">PYRAX</span>
              <p className="text-stone-500 mt-3 text-sm max-w-md">
                PYRAX is a next-generation blockchain ecosystem. Access our official brand assets 
                and approved materials through our public library.
              </p>
            </div>
            <div>
              <h4 className="font-medium text-stone-200 mb-4">Resources</h4>
              <ul className="space-y-2 text-sm">
                <li>
                  <Link href="/media-kit" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Media Kit
                  </Link>
                </li>
                <li>
                  <Link href="/public-library" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Public Library
                  </Link>
                </li>
                <li>
                  <a href="https://pyrax.org" target="_blank" rel="noopener noreferrer" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Main Website
                  </a>
                </li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-stone-200 mb-4">Legal</h4>
              <ul className="space-y-2 text-sm">
                <li>
                  <Link href="#" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Terms of Use
                  </Link>
                </li>
                <li>
                  <Link href="#" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Privacy Policy
                  </Link>
                </li>
                <li>
                  <Link href="#" className="text-stone-500 hover:text-stone-300 transition-colors">
                    Brand Guidelines
                  </Link>
                </li>
              </ul>
            </div>
          </div>
          <div className="border-t border-stone-800 mt-8 pt-8 text-center text-stone-600 text-sm">
            © {new Date().getFullYear()} PYRAX Blockchain. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  );
}
