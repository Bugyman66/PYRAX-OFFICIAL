import { getSession } from '@/lib/auth';
import { redirect } from 'next/navigation';
import { Sidebar } from '@/components/layout/Sidebar';
import { Header } from '@/components/layout/Header';

export default async function InternalLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const session = await getSession();
  
  if (!session) {
    redirect('/auth/login');
  }
  
  if (!session.isInternal) {
    redirect('/me/submissions');
  }

  return (
    <div className="min-h-screen bg-stone-950">
      <Sidebar user={session} />
      <div className="lg:pl-64">
        <Header user={session} />
        <main className="p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
