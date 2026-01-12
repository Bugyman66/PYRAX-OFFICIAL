import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { exchangeCodeForTokens } from '@/lib/drive';
import { prisma } from '@/lib/prisma';

export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.redirect(new URL('/auth/login?error=unauthorized', request.url));
    }

    if (session.role !== 'OrgAdmin') {
      return NextResponse.redirect(new URL('/app?error=forbidden', request.url));
    }

    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const error = searchParams.get('error');
    const state = searchParams.get('state');

    if (error) {
      console.error('Google OAuth error:', error);
      return NextResponse.redirect(
        new URL(`/app/admin/integrations?error=${encodeURIComponent(error)}`, request.url)
      );
    }

    if (!code) {
      return NextResponse.redirect(
        new URL('/app/admin/integrations?error=no_code', request.url)
      );
    }

    if (state) {
      try {
        const stateData = JSON.parse(Buffer.from(state, 'base64').toString('utf-8'));
        if (stateData.oderId !== session.id) {
          return NextResponse.redirect(
            new URL('/app/admin/integrations?error=invalid_state', request.url)
          );
        }
      } catch {
        console.warn('Failed to parse state, continuing anyway');
      }
    }

    const tokens = await exchangeCodeForTokens(code);

    await prisma.driveIntegration.updateMany({
      where: { isConnected: true },
      data: { isConnected: false },
    });

    const folderId = await createOrGetRootFolder(tokens.accessToken);

    await prisma.driveIntegration.create({
      data: {
        accessToken: tokens.accessToken,
        refreshToken: tokens.refreshToken,
        tokenExpiresAt: tokens.expiresAt,
        rootFolderId: folderId,
        isConnected: true,
        connectedByEmail: tokens.email,
      },
    });

    return NextResponse.redirect(
      new URL('/app/admin/integrations?success=connected', request.url)
    );
  } catch (error) {
    console.error('Google OAuth callback error:', error);
    
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return NextResponse.redirect(
      new URL(`/app/admin/integrations?error=${encodeURIComponent(errorMessage)}`, request.url)
    );
  }
}

async function createOrGetRootFolder(accessToken: string): Promise<string> {
  const { google } = await import('googleapis');
  
  const oauth2Client = new google.auth.OAuth2();
  oauth2Client.setCredentials({ access_token: accessToken });
  
  const drive = google.drive({ version: 'v3', auth: oauth2Client });

  const searchResponse = await drive.files.list({
    q: "name='PYRAX Proofing Hub' and mimeType='application/vnd.google-apps.folder' and trashed=false",
    fields: 'files(id,name)',
    spaces: 'drive',
  });

  if (searchResponse.data.files && searchResponse.data.files.length > 0) {
    return searchResponse.data.files[0].id!;
  }

  const createResponse = await drive.files.create({
    requestBody: {
      name: 'PYRAX Proofing Hub',
      mimeType: 'application/vnd.google-apps.folder',
    },
    fields: 'id',
  });

  if (!createResponse.data.id) {
    throw new Error('Failed to create root folder');
  }

  return createResponse.data.id;
}
