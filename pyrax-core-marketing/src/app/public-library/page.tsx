import { prisma } from '@/lib/prisma';
import { Download, Search, Grid, List, Image as ImageIcon, FileText, Video } from 'lucide-react';
import Link from 'next/link';

async function getPublicAssets() {
  try {
    return await prisma.publicAsset.findMany({
      where: {
        isPublished: true,
      },
      include: {
        proof: {
          include: {
            versions: {
              orderBy: { versionNumber: 'desc' },
              take: 1,
            },
            folder: {
              include: {
                department: true,
              },
            },
          },
        },
      },
      orderBy: { createdAt: 'desc' },
    });
  } catch {
    return [];
  }
}

function getFileIcon(mimeType?: string) {
  if (!mimeType) return FileText;
  if (mimeType.startsWith('image/')) return ImageIcon;
  if (mimeType.startsWith('video/')) return Video;
  if (mimeType === 'application/pdf') return FileText;
  return FileText;
}

export default async function PublicLibraryPage() {
  const assets = await getPublicAssets();

  return (
    <div className="min-h-screen bg-stone-950">
      <header className="border-b border-stone-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 pyrax-gradient rounded-xl flex items-center justify-center">
                <span className="text-white font-bold text-xl">P</span>
              </div>
              <div>
                <h1 className="text-2xl font-bold pyrax-gradient-text">Public Library</h1>
                <p className="text-stone-500">Download approved PYRAX assets</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <Link
                href="/media-kit"
                className="text-sm text-stone-400 hover:text-stone-50 transition-colors"
              >
                Brand Kit
              </Link>
              <Link
                href="/auth/login"
                className="px-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 hover:bg-stone-800 transition-colors"
              >
                Sign In
              </Link>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 mb-8">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
            <input
              type="text"
              placeholder="Search assets..."
              className="w-full pl-10 pr-4 py-2.5 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:border-transparent transition-all"
            />
          </div>
          <div className="flex items-center gap-2">
            <button className="p-2.5 bg-pyrax-500/10 text-pyrax-500 rounded-lg">
              <Grid className="w-5 h-5" />
            </button>
            <button className="p-2.5 text-stone-500 hover:text-stone-50 rounded-lg hover:bg-stone-900 transition-colors">
              <List className="w-5 h-5" />
            </button>
          </div>
        </div>

        {assets.length > 0 ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {assets.map((asset) => {
              const version = asset.proof.versions[0];
              const Icon = getFileIcon(version?.mimeType);
              
              return (
                <div
                  key={asset.id}
                  className="bg-stone-900 rounded-xl border border-stone-800 overflow-hidden group hover:border-stone-700 transition-colors"
                >
                  <div className="aspect-square bg-stone-800 flex items-center justify-center relative">
                    <Icon className="w-16 h-16 text-stone-700" />
                    {asset.downloadAllowed && (
                      <a
                        href={`/api/public/download/${asset.slug}`}
                        className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <div className="w-12 h-12 pyrax-gradient rounded-full flex items-center justify-center pyrax-glow">
                          <Download className="w-6 h-6 text-white" />
                        </div>
                      </a>
                    )}
                  </div>
                  <div className="p-4">
                    <h3 className="font-medium text-stone-50 truncate">
                      {asset.title || asset.proof.title}
                    </h3>
                    <p className="text-sm text-stone-500 mt-1">
                      {asset.proof.folder.department.name}
                    </p>
                    {asset.description && (
                      <p className="text-sm text-stone-400 mt-2 line-clamp-2">
                        {asset.description}
                      </p>
                    )}
                    <div className="flex items-center justify-between mt-4">
                      <span className="text-xs text-stone-500">
                        {version?.mimeType?.split('/')[1]?.toUpperCase() || 'FILE'}
                      </span>
                      {asset.downloadAllowed && (
                        <a
                          href={`/api/public/download/${asset.slug}`}
                          className="inline-flex items-center gap-1.5 text-sm text-pyrax-500 hover:text-pyrax-400 transition-colors"
                        >
                          <Download className="w-4 h-4" />
                          Download
                        </a>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="bg-stone-900 rounded-xl p-16 border border-stone-800 text-center">
            <ImageIcon className="w-20 h-20 text-stone-700 mx-auto mb-6" />
            <h2 className="text-xl font-semibold text-stone-50 mb-2">No Assets Available Yet</h2>
            <p className="text-stone-400 max-w-md mx-auto">
              Our team is working on adding approved assets to the public library. 
              Check back soon or visit our brand kit for guidelines.
            </p>
            <Link
              href="/media-kit"
              className="mt-6 inline-flex items-center gap-2 px-6 py-3 pyrax-gradient text-white font-semibold rounded-lg hover:opacity-90 transition-opacity pyrax-glow"
            >
              View Brand Kit
            </Link>
          </div>
        )}
      </main>

      <footer className="border-t border-stone-800 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <p className="text-stone-500 text-sm">
              © 2026 PYRAX Blockchain. All rights reserved.
            </p>
            <div className="flex items-center gap-6">
              <Link href="/media-kit" className="text-sm text-stone-400 hover:text-stone-50 transition-colors">
                Brand Kit
              </Link>
              <a href="https://pyrax.org" className="text-sm text-stone-400 hover:text-stone-50 transition-colors">
                PYRAX.org
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
