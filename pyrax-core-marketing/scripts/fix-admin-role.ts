import { PrismaClient, UserRole } from '@prisma/client';

const prisma = new PrismaClient();

async function main() {
  // Update user role to OrgAdmin
  const user = await prisma.user.update({
    where: { email: 'shawn.wilson@pyrax.org' },
    data: { role: UserRole.OrgAdmin }
  });
  
  console.log('✅ Updated user:', user.email);
  console.log('   Role:', user.role);
  
  // Clear all sessions for this user so they get a fresh session with correct role
  const deleted = await prisma.session.deleteMany({
    where: { userId: user.id }
  });
  
  console.log('🔄 Cleared', deleted.count, 'session(s)');
  console.log('');
  console.log('⚠️  Please log out and log back in to get the updated role!');
}

main()
  .then(() => prisma.$disconnect())
  .catch(async (e) => {
    console.error(e);
    await prisma.$disconnect();
    process.exit(1);
  });
