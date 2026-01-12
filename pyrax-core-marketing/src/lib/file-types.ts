export type FileCategory = 'pdf' | 'image' | 'video' | 'design' | 'unknown';

export function getFileCategory(mimeType: string): FileCategory {
  if (mimeType === 'application/pdf') {
    return 'pdf';
  }
  
  if (mimeType.startsWith('image/')) {
    // Check if it's a design file disguised as image
    if (
      mimeType.includes('photoshop') ||
      mimeType.includes('psd') ||
      mimeType.includes('xcf')
    ) {
      return 'design';
    }
    return 'image';
  }
  
  if (mimeType.startsWith('video/')) {
    return 'video';
  }
  
  // Design/source files
  if (
    mimeType.includes('illustrator') ||
    mimeType.includes('photoshop') ||
    mimeType.includes('indesign') ||
    mimeType.includes('sketch') ||
    mimeType.includes('figma') ||
    mimeType.includes('coreldraw') ||
    mimeType.includes('aftereffects') ||
    mimeType.includes('premiere') ||
    mimeType.includes('xd') ||
    mimeType.includes('afdesign') ||
    mimeType.includes('afphoto') ||
    mimeType.includes('krita') ||
    mimeType.includes('clip-studio') ||
    mimeType === 'application/postscript' ||
    mimeType.includes('eps')
  ) {
    return 'design';
  }
  
  return 'unknown';
}

export function getFileExtension(fileName: string): string {
  const parts = fileName.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

export function isViewableInBrowser(mimeType: string): boolean {
  const category = getFileCategory(mimeType);
  
  // PDF, images, and videos can be viewed in browser
  if (category === 'pdf' || category === 'image' || category === 'video') {
    // Exclude RAW camera formats
    if (mimeType.includes('x-raw') || mimeType.includes('x-canon') || 
        mimeType.includes('x-nikon') || mimeType.includes('x-sony') ||
        mimeType.includes('x-fuji') || mimeType.includes('x-panasonic') ||
        mimeType.includes('x-olympus') || mimeType.includes('dng')) {
      return false;
    }
    
    // Exclude HEIC/HEIF (limited browser support)
    if (mimeType === 'image/heic' || mimeType === 'image/heif') {
      return false;
    }
    
    return true;
  }
  
  return false;
}

export function getViewerType(mimeType: string): 'pdf' | 'image' | 'video' | 'unsupported' {
  if (mimeType === 'application/pdf') {
    return 'pdf';
  }
  
  if (mimeType.startsWith('image/') && isViewableInBrowser(mimeType)) {
    return 'image';
  }
  
  if (mimeType.startsWith('video/')) {
    return 'video';
  }
  
  return 'unsupported';
}

export function formatMimeType(mimeType: string): string {
  const mappings: Record<string, string> = {
    'application/pdf': 'PDF',
    'image/jpeg': 'JPEG',
    'image/png': 'PNG',
    'image/gif': 'GIF',
    'image/webp': 'WebP',
    'image/svg+xml': 'SVG',
    'image/tiff': 'TIFF',
    'image/bmp': 'BMP',
    'image/heic': 'HEIC',
    'image/heif': 'HEIF',
    'image/avif': 'AVIF',
    'video/mp4': 'MP4',
    'video/quicktime': 'MOV',
    'video/webm': 'WebM',
    'video/x-msvideo': 'AVI',
    'video/x-matroska': 'MKV',
    'application/vnd.adobe.photoshop': 'PSD',
    'image/vnd.adobe.photoshop': 'PSD',
    'application/vnd.adobe.illustrator': 'AI',
    'application/illustrator': 'AI',
    'application/vnd.adobe.indesign': 'INDD',
    'application/postscript': 'EPS',
  };
  
  return mappings[mimeType] || mimeType.split('/')[1]?.toUpperCase() || 'Unknown';
}
