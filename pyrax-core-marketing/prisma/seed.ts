import { PrismaClient, UserRole } from '@prisma/client';

const prisma = new PrismaClient();

async function main() {
  console.log('🌱 Seeding database...');

  const graphicsDept = await prisma.department.upsert({
    where: { name: 'Graphics & Branding' },
    update: {},
    create: {
      name: 'Graphics & Branding',
    },
  });

  const contentDept = await prisma.department.upsert({
    where: { name: 'Content Creation' },
    update: {},
    create: {
      name: 'Content Creation',
    },
  });

  console.log('✅ Departments created');

  const adminUser = await prisma.user.upsert({
    where: { email: 'shawn.wilson@pyrax.org' },
    update: {},
    create: {
      email: 'shawn.wilson@pyrax.org',
      name: 'Shawn Wilson',
      role: UserRole.OrgAdmin,
      isInternal: true,
    },
  });

  console.log('✅ Admin user created');

  const graphicsWorkflow = await prisma.workflowTemplate.upsert({
    where: {
      id: 'graphics-standard-workflow',
    },
    update: {},
    create: {
      id: 'graphics-standard-workflow',
      departmentId: graphicsDept.id,
      name: 'Standard Design Review',
      description: 'Standard workflow for design assets requiring review',
      isDefault: true,
      stepsJson: [
        {
          name: 'Initial Review',
          allowedRoles: ['Employee', 'DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'any',
        },
        {
          name: 'Department Approval',
          allowedRoles: ['DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'all',
        },
        {
          name: 'Final Sign-off',
          allowedRoles: ['OrgAdmin', 'DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'any',
        },
      ],
      finalRuleJson: {
        type: 'single',
        description: 'Single approver required for final sign-off',
      },
    },
  });

  const contentWorkflow = await prisma.workflowTemplate.upsert({
    where: {
      id: 'content-standard-workflow',
    },
    update: {},
    create: {
      id: 'content-standard-workflow',
      departmentId: contentDept.id,
      name: 'Content Review Process',
      description: 'Standard workflow for content requiring editorial review',
      isDefault: true,
      stepsJson: [
        {
          name: 'Editorial Review',
          allowedRoles: ['Employee', 'DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'any',
        },
        {
          name: 'Content Lead Approval',
          allowedRoles: ['DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'all',
        },
      ],
      finalRuleJson: {
        type: 'single',
        description: 'Content Lead approval required',
      },
    },
  });

  const externalIntakeWorkflow = await prisma.workflowTemplate.upsert({
    where: {
      id: 'external-intake-workflow',
    },
    update: {},
    create: {
      id: 'external-intake-workflow',
      departmentId: graphicsDept.id,
      name: 'External Submission Intake',
      description: 'Workflow for processing external content submissions',
      isDefault: false,
      stepsJson: [
        {
          name: 'Initial Screening',
          allowedRoles: ['Employee', 'DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'any',
        },
        {
          name: 'Department Review',
          allowedRoles: ['DepartmentHead'],
          allowedUsers: [],
          approvalRule: 'any',
        },
      ],
      finalRuleJson: {
        type: 'single',
        description: 'Department head approval required for external submissions',
      },
    },
  });

  console.log('✅ Workflow templates created');

  const brandFolder = await prisma.folder.upsert({
    where: {
      id: 'brand-assets-folder',
    },
    update: {},
    create: {
      id: 'brand-assets-folder',
      departmentId: graphicsDept.id,
      name: 'Brand Assets',
      description: 'Official PYRAX brand assets and guidelines',
      createdById: adminUser.id,
    },
  });

  const marketingFolder = await prisma.folder.upsert({
    where: {
      id: 'marketing-folder',
    },
    update: {},
    create: {
      id: 'marketing-folder',
      departmentId: contentDept.id,
      name: 'Marketing Materials',
      description: 'Marketing campaigns and promotional content',
      createdById: adminUser.id,
    },
  });

  console.log('✅ Sample folders created');

  console.log('');
  console.log('🎉 Database seeded successfully!');
  console.log('');
  console.log('📋 Created:');
  console.log('   - 2 Departments: Graphics & Branding, Content Creation');
  console.log('   - 1 Admin User:');
  console.log('     • shawn.wilson@pyrax.org (OrgAdmin)');
  console.log('   - 3 Workflow Templates');
  console.log('   - 2 Sample Folders');
  console.log('');
  console.log('🔐 To login, request a magic link for shawn.wilson@pyrax.org');
}

main()
  .then(async () => {
    await prisma.$disconnect();
  })
  .catch(async (e) => {
    console.error(e);
    await prisma.$disconnect();
    process.exit(1);
  });
