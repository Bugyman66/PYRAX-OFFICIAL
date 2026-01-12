import { prisma } from '@/lib/prisma';
import { Download, ExternalLink, Palette, Type, Image as ImageIcon } from 'lucide-react';
import Link from 'next/link';

async function getPublicAssets() {
  try {
    return await prisma.publicAsset.findMany({
      where: {
        isPublished: true,
        mediaKitSection: { not: null },
      },
      include: {
        proof: {
          include: {
            versions: {
              where: { kind: 'Rendition' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
            },
          },
        },
      },
      orderBy: [
        { mediaKitSection: 'asc' },
        { displayOrder: 'asc' },
      ],
    });
  } catch {
    return [];
  }
}

export default async function MediaKitPage() {
  const assets = await getPublicAssets();

  const groupedAssets = assets.reduce((acc, asset) => {
    const section = asset.mediaKitSection || 'Other';
    if (!acc[section]) acc[section] = [];
    acc[section].push(asset);
    return acc;
  }, {} as Record<string, typeof assets>);

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
                <h1 className="text-2xl font-bold pyrax-gradient-text">PYRAX Brand Kit</h1>
                <p className="text-stone-500">Official brand guidelines and assets</p>
              </div>
            </div>
            <Link
              href="/auth/login"
              className="px-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 hover:bg-stone-800 transition-colors"
            >
              Sign In
            </Link>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <section className="mb-16">
          <div className="bg-gradient-to-r from-pyrax-500/10 to-pyrax-700/10 rounded-2xl p-8 border border-pyrax-500/20">
            <h2 className="text-3xl font-bold text-stone-50 mb-4">Welcome to the PYRAX Brand Kit</h2>
            <p className="text-stone-400 text-lg max-w-3xl">
              This media kit contains official PYRAX Blockchain brand assets, guidelines, and resources. 
              All assets are approved for use by partners, media, and community members following our brand guidelines.
            </p>
          </div>
        </section>

        <section className="mb-16">
          <h2 className="text-xl font-semibold text-stone-50 mb-6 flex items-center gap-3">
            <Palette className="w-6 h-6 text-pyrax-500" />
            Brand Colors
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {[
              { name: 'PYRAX Orange', hex: '#ff6b35', desc: 'Primary brand color' },
              { name: 'PYRAX Rust', hex: '#ea580c', desc: 'Secondary brand color' },
              { name: 'Stone Dark', hex: '#0c0a09', desc: 'Background color' },
              { name: 'Stone Gray', hex: '#1c1917', desc: 'Secondary background' },
            ].map((color) => (
              <div key={color.hex} className="bg-stone-900 rounded-xl p-4 border border-stone-800">
                <div
                  className="w-full h-20 rounded-lg mb-3"
                  style={{ backgroundColor: color.hex }}
                />
                <p className="font-medium text-stone-50">{color.name}</p>
                <p className="text-sm text-pyrax-500 font-mono">{color.hex}</p>
                <p className="text-xs text-stone-500 mt-1">{color.desc}</p>
              </div>
            ))}
          </div>
        </section>

        <section className="mb-16">
          <h2 className="text-xl font-semibold text-stone-50 mb-6 flex items-center gap-3">
            <Type className="w-6 h-6 text-pyrax-500" />
            Typography
          </h2>
          <div className="bg-stone-900 rounded-xl p-6 border border-stone-800">
            <div className="space-y-6">
              <div>
                <p className="text-sm text-stone-500 mb-2">Primary Font</p>
                <p className="text-4xl font-bold text-stone-50">Inter</p>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div>
                  <p className="text-sm text-stone-500 mb-2">Headings</p>
                  <p className="text-2xl font-bold text-stone-50">Bold 700</p>
                </div>
                <div>
                  <p className="text-sm text-stone-500 mb-2">Body</p>
                  <p className="text-lg text-stone-50">Regular 400</p>
                </div>
                <div>
                  <p className="text-sm text-stone-500 mb-2">Captions</p>
                  <p className="text-sm text-stone-400">Medium 500</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="mb-16">
          <h2 className="text-xl font-semibold text-stone-50 mb-6 flex items-center gap-3">
            <ImageIcon className="w-6 h-6 text-pyrax-500" />
            Logo Assets
          </h2>
          
          {Object.keys(groupedAssets).length > 0 ? (
            <div className="space-y-8">
              {Object.entries(groupedAssets).map(([section, sectionAssets]) => (
                <div key={section}>
                  <h3 className="text-lg font-medium text-stone-50 mb-4">{section}</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {sectionAssets.map((asset) => (
                      <div
                        key={asset.id}
                        className="bg-stone-900 rounded-xl border border-stone-800 overflow-hidden"
                      >
                        <div className="aspect-video bg-stone-800 flex items-center justify-center">
                          <ImageIcon className="w-12 h-12 text-stone-700" />
                        </div>
                        <div className="p-4">
                          <h4 className="font-medium text-stone-50">{asset.title || asset.proof.title}</h4>
                          {asset.description && (
                            <p className="text-sm text-stone-500 mt-1">{asset.description}</p>
                          )}
                          {asset.downloadAllowed && (
                            <a
                              href={`/api/public/download/${asset.slug}`}
                              className="mt-3 inline-flex items-center gap-2 px-3 py-1.5 bg-pyrax-500/10 text-pyrax-500 rounded-lg text-sm hover:bg-pyrax-500/20 transition-colors"
                            >
                              <Download className="w-4 h-4" />
                              Download
                            </a>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="bg-stone-900 rounded-xl p-12 border border-stone-800 text-center">
              <ImageIcon className="w-16 h-16 text-stone-700 mx-auto mb-4" />
              <p className="text-stone-400">Logo assets will be available soon.</p>
              <p className="text-sm text-stone-500 mt-2">
                Check back later for downloadable brand assets.
              </p>
            </div>
          )}
        </section>

        <section className="mb-16">
          <h2 className="text-xl font-semibold text-stone-50 mb-6">Usage Guidelines</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-stone-900 rounded-xl p-6 border border-stone-800">
              <h3 className="text-lg font-medium text-green-400 mb-4">✓ Do</h3>
              <ul className="space-y-2 text-stone-400">
                <li>• Use official logo files provided in this kit</li>
                <li>• Maintain minimum clear space around logos</li>
                <li>• Use approved brand colors</li>
                <li>• Follow typography guidelines</li>
              </ul>
            </div>
            <div className="bg-stone-900 rounded-xl p-6 border border-stone-800">
              <h3 className="text-lg font-medium text-red-400 mb-4">✗ Don&apos;t</h3>
              <ul className="space-y-2 text-stone-400">
                <li>• Modify logo colors or proportions</li>
                <li>• Add effects or shadows to logos</li>
                <li>• Use low-resolution assets</li>
                <li>• Create derivative logos</li>
              </ul>
            </div>
          </div>
        </section>

        <section>
          <div className="bg-stone-900 rounded-xl p-8 border border-stone-800 text-center">
            <h2 className="text-xl font-semibold text-stone-50 mb-4">Need Custom Assets?</h2>
            <p className="text-stone-400 mb-6 max-w-2xl mx-auto">
              If you need custom branded materials or have questions about brand usage, 
              please submit a request through our submission portal.
            </p>
            <Link
              href="/submit"
              className="inline-flex items-center gap-2 px-6 py-3 pyrax-gradient text-white font-semibold rounded-lg hover:opacity-90 transition-opacity pyrax-glow"
            >
              Submit Request
              <ExternalLink className="w-4 h-4" />
            </Link>
          </div>
        </section>
      </main>

      <footer className="border-t border-stone-800 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <p className="text-stone-500 text-sm">
              © 2026 PYRAX Blockchain. All rights reserved.
            </p>
            <div className="flex items-center gap-6">
              <Link href="/public-library" className="text-sm text-stone-400 hover:text-stone-50 transition-colors">
                Public Library
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
