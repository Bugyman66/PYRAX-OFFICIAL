import { cookies } from 'next/headers';
import { prisma } from './prisma';
import { v4 as uuidv4 } from 'uuid';
import jwt from 'jsonwebtoken';
import { UserRole } from '@prisma/client';

const JWT_SECRET = process.env.NEXTAUTH_SECRET || 'fallback-secret-change-me';
const INTERNAL_DOMAIN = process.env.INTERNAL_EMAIL_DOMAIN || 'pyrax.org';
const MAGIC_LINK_EXPIRY = parseInt(process.env.MAGIC_LINK_EXPIRY_MINUTES || '15');
const SESSION_EXPIRY_DAYS = parseInt(process.env.SESSION_EXPIRY_DAYS || '30');

export interface AuthUser {
  id: string;
  email: string;
  name: string | null;
  role: UserRole;
  departmentId: string | null;
  isInternal: boolean;
}

export function isInternalEmail(email: string): boolean {
  return email.toLowerCase().endsWith(`@${INTERNAL_DOMAIN.toLowerCase()}`);
}

export async function createMagicLinkToken(email: string): Promise<string | null> {
  const isInternal = isInternalEmail(email);
  
  let user = await prisma.user.findUnique({ where: { email: email.toLowerCase() } });
  
  if (!user) {
    if (isInternal) {
      user = await prisma.user.create({
        data: {
          email: email.toLowerCase(),
          role: UserRole.Employee,
          isInternal: true,
        },
      });
    } else {
      user = await prisma.user.create({
        data: {
          email: email.toLowerCase(),
          role: UserRole.PublicSubmitter,
          isInternal: false,
        },
      });
    }
  }
  
  if (user.isInternal !== isInternal) {
    return null;
  }

  const token = uuidv4();
  const expiresAt = new Date(Date.now() + MAGIC_LINK_EXPIRY * 60 * 1000);
  
  await prisma.magicLinkToken.create({
    data: {
      token,
      userId: user.id,
      email: email.toLowerCase(),
      expiresAt,
    },
  });
  
  return token;
}

export async function verifyMagicLinkToken(token: string): Promise<AuthUser | null> {
  const magicLink = await prisma.magicLinkToken.findUnique({
    where: { token },
    include: { user: true },
  });
  
  if (!magicLink) return null;
  if (magicLink.usedAt) return null;
  if (magicLink.expiresAt < new Date()) return null;
  
  await prisma.magicLinkToken.update({
    where: { id: magicLink.id },
    data: { usedAt: new Date() },
  });
  
  return {
    id: magicLink.user.id,
    email: magicLink.user.email,
    name: magicLink.user.name,
    role: magicLink.user.role,
    departmentId: magicLink.user.departmentId,
    isInternal: magicLink.user.isInternal,
  };
}

export async function createSession(userId: string): Promise<string> {
  const token = jwt.sign({ userId }, JWT_SECRET, { expiresIn: `${SESSION_EXPIRY_DAYS}d` });
  const expiresAt = new Date(Date.now() + SESSION_EXPIRY_DAYS * 24 * 60 * 60 * 1000);
  
  await prisma.session.create({
    data: {
      userId,
      token,
      expiresAt,
    },
  });
  
  return token;
}

export async function getSession(): Promise<AuthUser | null> {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('session')?.value;
  
  if (!sessionToken) return null;
  
  try {
    jwt.verify(sessionToken, JWT_SECRET);
    
    const session = await prisma.session.findUnique({
      where: { token: sessionToken },
      include: { user: true },
    });
    
    if (!session) return null;
    if (session.expiresAt < new Date()) {
      await prisma.session.delete({ where: { id: session.id } });
      return null;
    }
    
    return {
      id: session.user.id,
      email: session.user.email,
      name: session.user.name,
      role: session.user.role,
      departmentId: session.user.departmentId,
      isInternal: session.user.isInternal,
    };
  } catch {
    return null;
  }
}

export async function destroySession(): Promise<void> {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('session')?.value;
  
  if (sessionToken) {
    await prisma.session.deleteMany({ where: { token: sessionToken } });
  }
}

export function canAccessInternalRoutes(user: AuthUser | null): boolean {
  if (!user) return false;
  return user.isInternal;
}

export function canAccessAdminRoutes(user: AuthUser | null): boolean {
  if (!user) return false;
  return user.role === UserRole.OrgAdmin;
}

export function canAccessDepartmentRoutes(user: AuthUser | null): boolean {
  if (!user) return false;
  return user.role === UserRole.OrgAdmin || user.role === UserRole.DepartmentHead;
}

export function canManageWorkflows(user: AuthUser | null, departmentId: string): boolean {
  if (!user) return false;
  if (user.role === UserRole.OrgAdmin) return true;
  if (user.role === UserRole.DepartmentHead && user.departmentId === departmentId) return true;
  return false;
}

export function canPublishAssets(user: AuthUser | null, departmentId: string): boolean {
  return canManageWorkflows(user, departmentId);
}
