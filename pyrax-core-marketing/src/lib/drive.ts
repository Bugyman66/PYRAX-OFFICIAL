import { google, drive_v3 } from 'googleapis';
import { Readable } from 'stream';
import { prisma } from './prisma';

interface UploadOptions {
  buffer: Buffer;
  fileName: string;
  mimeType: string;
  folderId?: string;
  description?: string;
}

interface FileMetadata {
  id: string;
  name: string;
  mimeType: string;
  size: number;
  createdTime: string;
  modifiedTime: string;
  webViewLink?: string;
  thumbnailLink?: string;
}

interface DriveFolder {
  id: string;
  name: string;
  createdTime: string;
}

interface DriveConnectionStatus {
  connected: boolean;
  userEmail?: string;
  rootFolderId?: string;
  rootFolderName?: string;
  error?: string;
}

class DriveServiceError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500
  ) {
    super(message);
    this.name = 'DriveServiceError';
  }
}

const SCOPES = [
  'https://www.googleapis.com/auth/drive.file',
  'https://www.googleapis.com/auth/drive.readonly',
  'https://www.googleapis.com/auth/userinfo.email',
];

function getRedirectUri(): string {
  const env = process.env.NODE_ENV;
  
  if (env === 'development') {
    return 'http://localhost:3000/api/auth/google/callback';
  }
  
  // For testnet/mainnet (production), use the configured URI or default
  return process.env.GOOGLE_REDIRECT_URI || 'https://marketing.pyrax.org/api/auth/google/callback';
}

function getOAuth2Client() {
  const clientId = process.env.GOOGLE_CLIENT_ID;
  const clientSecret = process.env.GOOGLE_CLIENT_SECRET;
  const redirectUri = getRedirectUri();

  if (!clientId || !clientSecret) {
    throw new DriveServiceError(
      'Google OAuth credentials not configured',
      'OAUTH_NOT_CONFIGURED',
      503
    );
  }

  return new google.auth.OAuth2(clientId, clientSecret, redirectUri);
}

export function getAuthUrl(state?: string): string {
  const oauth2Client = getOAuth2Client();
  
  return oauth2Client.generateAuthUrl({
    access_type: 'offline',
    scope: SCOPES,
    prompt: 'consent',
    state: state || '',
  });
}

export async function exchangeCodeForTokens(code: string): Promise<{
  accessToken: string;
  refreshToken: string;
  expiresAt: Date;
  email: string;
}> {
  const oauth2Client = getOAuth2Client();
  
  const { tokens } = await oauth2Client.getToken(code);
  
  if (!tokens.refresh_token) {
    throw new DriveServiceError(
      'No refresh token received. Please revoke access and try again.',
      'NO_REFRESH_TOKEN',
      400
    );
  }

  oauth2Client.setCredentials(tokens);
  
  const oauth2 = google.oauth2({ version: 'v2', auth: oauth2Client });
  const userInfo = await oauth2.userinfo.get();
  
  return {
    accessToken: tokens.access_token || '',
    refreshToken: tokens.refresh_token,
    expiresAt: new Date(tokens.expiry_date || Date.now() + 3600000),
    email: userInfo.data.email || '',
  };
}

class DriveService {
  private drive: drive_v3.Drive | null = null;
  private oauth2Client: ReturnType<typeof getOAuth2Client> | null = null;
  private rootFolderId: string = '';

  private async getIntegration() {
    const integration = await prisma.driveIntegration.findFirst({
      where: { isConnected: true },
      orderBy: { createdAt: 'desc' },
    });
    return integration;
  }

  private async refreshTokenIfNeeded(integration: NonNullable<Awaited<ReturnType<typeof this.getIntegration>>>) {
    if (!integration.tokenExpiresAt || integration.tokenExpiresAt > new Date()) {
      return integration;
    }

    const oauth2Client = getOAuth2Client();
    oauth2Client.setCredentials({
      refresh_token: integration.refreshToken,
    });

    const { credentials } = await oauth2Client.refreshAccessToken();
    
    const updated = await prisma.driveIntegration.update({
      where: { id: integration.id },
      data: {
        accessToken: credentials.access_token,
        tokenExpiresAt: credentials.expiry_date ? new Date(credentials.expiry_date) : null,
      },
    });

    return updated;
  }

  private async initialize(): Promise<void> {
    const integration = await this.getIntegration();
    
    if (!integration) {
      throw new DriveServiceError(
        'Google Drive not connected. Please connect via Admin > Integrations.',
        'DRIVE_NOT_CONNECTED',
        503
      );
    }

    const refreshedIntegration = await this.refreshTokenIfNeeded(integration);
    
    this.oauth2Client = getOAuth2Client();
    this.oauth2Client.setCredentials({
      access_token: refreshedIntegration.accessToken,
      refresh_token: refreshedIntegration.refreshToken,
    });

    this.drive = google.drive({ version: 'v3', auth: this.oauth2Client });
    this.rootFolderId = refreshedIntegration.rootFolderId;
  }

  private async getDrive(): Promise<drive_v3.Drive> {
    await this.initialize();
    if (!this.drive) {
      throw new DriveServiceError('Drive service not initialized', 'DRIVE_NOT_INITIALIZED', 500);
    }
    return this.drive;
  }

  async isConfigured(): Promise<boolean> {
    try {
      const integration = await this.getIntegration();
      return !!integration;
    } catch {
      return false;
    }
  }

  async getConnectionStatus(): Promise<DriveConnectionStatus> {
    try {
      const integration = await this.getIntegration();
      
      if (!integration) {
        return {
          connected: false,
          error: 'Google Drive not connected',
        };
      }

      await this.initialize();
      const drive = await this.getDrive();

      const folderResponse = await drive.files.get({
        fileId: this.rootFolderId,
        fields: 'id,name',
      });

      return {
        connected: true,
        userEmail: integration.connectedByEmail,
        rootFolderId: this.rootFolderId,
        rootFolderName: folderResponse.data.name || undefined,
      };
    } catch (error) {
      return {
        connected: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  async uploadFile(options: UploadOptions): Promise<FileMetadata> {
    const drive = await this.getDrive();
    const { buffer, fileName, mimeType, folderId, description } = options;

    const targetFolderId = folderId || this.rootFolderId;

    try {
      const readable = new Readable();
      readable.push(buffer);
      readable.push(null);

      const response = await drive.files.create({
        requestBody: {
          name: fileName,
          mimeType,
          parents: [targetFolderId],
          description,
        },
        media: {
          mimeType,
          body: readable,
        },
        fields: 'id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink',
      });

      if (!response.data.id) {
        throw new DriveServiceError('Upload succeeded but no file ID returned', 'UPLOAD_NO_ID', 500);
      }

      return {
        id: response.data.id,
        name: response.data.name || fileName,
        mimeType: response.data.mimeType || mimeType,
        size: parseInt(response.data.size || '0', 10),
        createdTime: response.data.createdTime || new Date().toISOString(),
        modifiedTime: response.data.modifiedTime || new Date().toISOString(),
        webViewLink: response.data.webViewLink || undefined,
        thumbnailLink: response.data.thumbnailLink || undefined,
      };
    } catch (error) {
      if (error instanceof DriveServiceError) throw error;
      
      const message = error instanceof Error ? error.message : 'Unknown error';
      throw new DriveServiceError(`Failed to upload file: ${message}`, 'UPLOAD_FAILED', 500);
    }
  }

  async uploadFileResumable(options: UploadOptions): Promise<FileMetadata> {
    const drive = await this.getDrive();
    const { buffer, fileName, mimeType, folderId, description } = options;

    const targetFolderId = folderId || this.rootFolderId;

    try {
      const readable = new Readable();
      readable.push(buffer);
      readable.push(null);

      const response = await drive.files.create({
        requestBody: {
          name: fileName,
          mimeType,
          parents: [targetFolderId],
          description,
        },
        media: {
          mimeType,
          body: readable,
        },
        fields: 'id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink',
      }, {
        onUploadProgress: (evt) => {
          if (process.env.NODE_ENV === 'development') {
            const progress = (evt.bytesRead / buffer.length) * 100;
            console.log(`Upload progress: ${progress.toFixed(1)}%`);
          }
        },
      });

      if (!response.data.id) {
        throw new DriveServiceError('Upload succeeded but no file ID returned', 'UPLOAD_NO_ID', 500);
      }

      return {
        id: response.data.id,
        name: response.data.name || fileName,
        mimeType: response.data.mimeType || mimeType,
        size: parseInt(response.data.size || '0', 10),
        createdTime: response.data.createdTime || new Date().toISOString(),
        modifiedTime: response.data.modifiedTime || new Date().toISOString(),
        webViewLink: response.data.webViewLink || undefined,
        thumbnailLink: response.data.thumbnailLink || undefined,
      };
    } catch (error) {
      if (error instanceof DriveServiceError) throw error;
      
      const message = error instanceof Error ? error.message : 'Unknown error';
      throw new DriveServiceError(`Failed to upload file: ${message}`, 'UPLOAD_FAILED', 500);
    }
  }

  async downloadFile(fileId: string): Promise<{ stream: Readable; metadata: FileMetadata }> {
    const drive = await this.getDrive();

    try {
      const metadataResponse = await drive.files.get({
        fileId,
        fields: 'id,name,mimeType,size,createdTime,modifiedTime',
      });

      const response = await drive.files.get(
        { fileId, alt: 'media' },
        { responseType: 'stream' }
      );

      return {
        stream: response.data as unknown as Readable,
        metadata: {
          id: metadataResponse.data.id || fileId,
          name: metadataResponse.data.name || 'unknown',
          mimeType: metadataResponse.data.mimeType || 'application/octet-stream',
          size: parseInt(metadataResponse.data.size || '0', 10),
          createdTime: metadataResponse.data.createdTime || '',
          modifiedTime: metadataResponse.data.modifiedTime || '',
        },
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      
      if (message.includes('404') || message.includes('not found')) {
        throw new DriveServiceError('File not found', 'FILE_NOT_FOUND', 404);
      }
      
      throw new DriveServiceError(`Failed to download file: ${message}`, 'DOWNLOAD_FAILED', 500);
    }
  }

  async getFileMetadata(fileId: string): Promise<FileMetadata> {
    const drive = await this.getDrive();

    try {
      const response = await drive.files.get({
        fileId,
        fields: 'id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink',
      });

      return {
        id: response.data.id || fileId,
        name: response.data.name || 'unknown',
        mimeType: response.data.mimeType || 'application/octet-stream',
        size: parseInt(response.data.size || '0', 10),
        createdTime: response.data.createdTime || '',
        modifiedTime: response.data.modifiedTime || '',
        webViewLink: response.data.webViewLink || undefined,
        thumbnailLink: response.data.thumbnailLink || undefined,
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      
      if (message.includes('404') || message.includes('not found')) {
        throw new DriveServiceError('File not found', 'FILE_NOT_FOUND', 404);
      }
      
      throw new DriveServiceError(`Failed to get file metadata: ${message}`, 'METADATA_FAILED', 500);
    }
  }

  async deleteFile(fileId: string): Promise<void> {
    const drive = await this.getDrive();

    try {
      await drive.files.delete({ fileId });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      
      if (message.includes('404') || message.includes('not found')) {
        throw new DriveServiceError('File not found', 'FILE_NOT_FOUND', 404);
      }
      
      throw new DriveServiceError(`Failed to delete file: ${message}`, 'DELETE_FAILED', 500);
    }
  }

  async createFolder(name: string, parentId?: string): Promise<DriveFolder> {
    const drive = await this.getDrive();
    const targetParentId = parentId || this.rootFolderId;

    try {
      const response = await drive.files.create({
        requestBody: {
          name,
          mimeType: 'application/vnd.google-apps.folder',
          parents: [targetParentId],
        },
        fields: 'id,name,createdTime',
      });

      if (!response.data.id) {
        throw new DriveServiceError('Folder creation succeeded but no ID returned', 'FOLDER_NO_ID', 500);
      }

      return {
        id: response.data.id,
        name: response.data.name || name,
        createdTime: response.data.createdTime || new Date().toISOString(),
      };
    } catch (error) {
      if (error instanceof DriveServiceError) throw error;
      
      const message = error instanceof Error ? error.message : 'Unknown error';
      throw new DriveServiceError(`Failed to create folder: ${message}`, 'FOLDER_CREATE_FAILED', 500);
    }
  }

  async listFiles(folderId?: string, pageToken?: string): Promise<{
    files: FileMetadata[];
    nextPageToken?: string;
  }> {
    const drive = await this.getDrive();
    const targetFolderId = folderId || this.rootFolderId;

    try {
      const response = await drive.files.list({
        q: `'${targetFolderId}' in parents and trashed = false`,
        fields: 'nextPageToken,files(id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink)',
        pageSize: 100,
        pageToken: pageToken || undefined,
        orderBy: 'createdTime desc',
      });

      const files: FileMetadata[] = (response.data.files || []).map((file) => ({
        id: file.id || '',
        name: file.name || 'unknown',
        mimeType: file.mimeType || 'application/octet-stream',
        size: parseInt(file.size || '0', 10),
        createdTime: file.createdTime || '',
        modifiedTime: file.modifiedTime || '',
        webViewLink: file.webViewLink || undefined,
        thumbnailLink: file.thumbnailLink || undefined,
      }));

      return {
        files,
        nextPageToken: response.data.nextPageToken || undefined,
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      throw new DriveServiceError(`Failed to list files: ${message}`, 'LIST_FAILED', 500);
    }
  }

  async moveFile(fileId: string, newFolderId: string): Promise<FileMetadata> {
    const drive = await this.getDrive();

    try {
      const file = await drive.files.get({
        fileId,
        fields: 'parents',
      });

      const previousParents = file.data.parents?.join(',') || '';

      const response = await drive.files.update({
        fileId,
        addParents: newFolderId,
        removeParents: previousParents,
        fields: 'id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink',
      });

      return {
        id: response.data.id || fileId,
        name: response.data.name || 'unknown',
        mimeType: response.data.mimeType || 'application/octet-stream',
        size: parseInt(response.data.size || '0', 10),
        createdTime: response.data.createdTime || '',
        modifiedTime: response.data.modifiedTime || '',
        webViewLink: response.data.webViewLink || undefined,
        thumbnailLink: response.data.thumbnailLink || undefined,
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      
      if (message.includes('404') || message.includes('not found')) {
        throw new DriveServiceError('File not found', 'FILE_NOT_FOUND', 404);
      }
      
      throw new DriveServiceError(`Failed to move file: ${message}`, 'MOVE_FAILED', 500);
    }
  }

  async copyFile(fileId: string, newName?: string, folderId?: string): Promise<FileMetadata> {
    const drive = await this.getDrive();
    const targetFolderId = folderId || this.rootFolderId;

    try {
      const response = await drive.files.copy({
        fileId,
        requestBody: {
          name: newName,
          parents: [targetFolderId],
        },
        fields: 'id,name,mimeType,size,createdTime,modifiedTime,webViewLink,thumbnailLink',
      });

      return {
        id: response.data.id || '',
        name: response.data.name || newName || 'unknown',
        mimeType: response.data.mimeType || 'application/octet-stream',
        size: parseInt(response.data.size || '0', 10),
        createdTime: response.data.createdTime || '',
        modifiedTime: response.data.modifiedTime || '',
        webViewLink: response.data.webViewLink || undefined,
        thumbnailLink: response.data.thumbnailLink || undefined,
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      
      if (message.includes('404') || message.includes('not found')) {
        throw new DriveServiceError('File not found', 'FILE_NOT_FOUND', 404);
      }
      
      throw new DriveServiceError(`Failed to copy file: ${message}`, 'COPY_FAILED', 500);
    }
  }

  getRootFolderId(): string {
    return this.rootFolderId;
  }
}

export const driveService = new DriveService();
export { DriveService, DriveServiceError };
export type { FileMetadata, DriveFolder, UploadOptions };
