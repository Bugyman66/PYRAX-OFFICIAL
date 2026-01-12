import { getSession } from '@/lib/auth';
import { redirect } from 'next/navigation';

export default async function AdminLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const session = await getSession();
  
  if (!session) {
    redirect('/auth/login');
  }
  
  if (session.role !== 'OrgAdmin') {
    redirect('/app');
  }

  return <>{children}</>;
}
