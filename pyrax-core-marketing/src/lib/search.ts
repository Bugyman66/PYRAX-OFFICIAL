import { prisma } from './prisma';
import { ProofStatus, UserRole, Prisma } from '@prisma/client';

export interface SearchFilters {
  query?: string;
  departmentId?: string;
  status?: ProofStatus;
  createdById?: string;
  dateFrom?: Date;
  dateTo?: Date;
}

export interface SearchResult {
  type: 'proof' | 'folder' | 'comment';
  id: string;
  title: string;
  description?: string;
  matchedField?: string;
  proof?: {
    id: string;
    title: string;
    status: ProofStatus;
  };
  folder?: {
    id: string;
    name: string;
  };
  createdBy?: {
    id: string;
    name: string | null;
    email: string;
  };
  createdAt: Date;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export class SearchService {
  async search(
    filters: SearchFilters,
    departmentId: string | null,
    role: UserRole,
    page = 1,
    pageSize = 20
  ): Promise<SearchResponse> {
    const results: SearchResult[] = [];
    const skip = (page - 1) * pageSize;

    const baseProofWhere = this.getBaseProofWhere(departmentId, role, filters);

    if (filters.query) {
      const searchTerm = filters.query.toLowerCase();

      const proofs = await prisma.proof.findMany({
        where: {
          ...baseProofWhere,
          OR: [
            { title: { contains: searchTerm, mode: 'insensitive' } },
            { description: { contains: searchTerm, mode: 'insensitive' } },
          ],
        },
        include: {
          folder: { select: { id: true, name: true } },
          createdBy: { select: { id: true, name: true, email: true } },
        },
        orderBy: { updatedAt: 'desc' },
        take: pageSize,
        skip,
      });

      for (const proof of proofs) {
        const matchedField = proof.title.toLowerCase().includes(searchTerm) ? 'title' : 'description';
        results.push({
          type: 'proof',
          id: proof.id,
          title: proof.title,
          description: proof.description || undefined,
          matchedField,
          proof: { id: proof.id, title: proof.title, status: proof.status },
          folder: proof.folder,
          createdBy: proof.createdBy,
          createdAt: proof.createdAt,
        });
      }

      const folders = await prisma.folder.findMany({
        where: {
          ...this.getBaseFolderWhere(departmentId, role),
          OR: [
            { name: { contains: searchTerm, mode: 'insensitive' } },
            { description: { contains: searchTerm, mode: 'insensitive' } },
          ],
        },
        include: {
          createdBy: { select: { id: true, name: true, email: true } },
        },
        orderBy: { updatedAt: 'desc' },
        take: 10,
      });

      for (const folder of folders) {
        results.push({
          type: 'folder',
          id: folder.id,
          title: folder.name,
          description: folder.description || undefined,
          matchedField: folder.name.toLowerCase().includes(searchTerm) ? 'name' : 'description',
          folder: { id: folder.id, name: folder.name },
          createdBy: folder.createdBy,
          createdAt: folder.createdAt,
        });
      }

      const comments = await prisma.comment.findMany({
        where: {
          body: { contains: searchTerm, mode: 'insensitive' },
          proofVersion: {
            proof: baseProofWhere,
          },
        },
        include: {
          createdBy: { select: { id: true, name: true, email: true } },
          proofVersion: {
            include: {
              proof: {
                select: { id: true, title: true, status: true },
                include: {
                  folder: { select: { id: true, name: true } },
                },
              },
            },
          },
        },
        orderBy: { createdAt: 'desc' },
        take: 10,
      });

      for (const comment of comments) {
        results.push({
          type: 'comment',
          id: comment.id,
          title: `Comment on "${comment.proofVersion.proof.title}"`,
          description: comment.body.substring(0, 150) + (comment.body.length > 150 ? '...' : ''),
          matchedField: 'body',
          proof: {
            id: comment.proofVersion.proof.id,
            title: comment.proofVersion.proof.title,
            status: comment.proofVersion.proof.status,
          },
          folder: comment.proofVersion.proof.folder,
          createdBy: comment.createdBy,
          createdAt: comment.createdAt,
        });
      }

      results.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
    } else {
      const proofs = await prisma.proof.findMany({
        where: baseProofWhere,
        include: {
          folder: { select: { id: true, name: true } },
          createdBy: { select: { id: true, name: true, email: true } },
        },
        orderBy: { updatedAt: 'desc' },
        take: pageSize,
        skip,
      });

      for (const proof of proofs) {
        results.push({
          type: 'proof',
          id: proof.id,
          title: proof.title,
          description: proof.description || undefined,
          proof: { id: proof.id, title: proof.title, status: proof.status },
          folder: proof.folder,
          createdBy: proof.createdBy,
          createdAt: proof.createdAt,
        });
      }
    }

    const total = await this.getTotalCount(filters, departmentId, role);

    return {
      results: results.slice(0, pageSize),
      total,
      page,
      pageSize,
      totalPages: Math.ceil(total / pageSize),
    };
  }

  async getSuggestions(
    query: string,
    departmentId: string | null,
    role: UserRole,
    limit = 5
  ): Promise<{ type: string; id: string; title: string }[]> {
    if (!query || query.length < 2) return [];

    const searchTerm = query.toLowerCase();
    const suggestions: { type: string; id: string; title: string }[] = [];

    const proofs = await prisma.proof.findMany({
      where: {
        ...this.getBaseProofWhere(departmentId, role, {}),
        title: { contains: searchTerm, mode: 'insensitive' },
      },
      select: { id: true, title: true },
      take: limit,
    });

    for (const proof of proofs) {
      suggestions.push({ type: 'proof', id: proof.id, title: proof.title });
    }

    if (suggestions.length < limit) {
      const folders = await prisma.folder.findMany({
        where: {
          ...this.getBaseFolderWhere(departmentId, role),
          name: { contains: searchTerm, mode: 'insensitive' },
        },
        select: { id: true, name: true },
        take: limit - suggestions.length,
      });

      for (const folder of folders) {
        suggestions.push({ type: 'folder', id: folder.id, title: folder.name });
      }
    }

    return suggestions;
  }

  private async getTotalCount(
    filters: SearchFilters,
    departmentId: string | null,
    role: UserRole
  ): Promise<number> {
    const baseWhere = this.getBaseProofWhere(departmentId, role, filters);

    if (filters.query) {
      const searchTerm = filters.query.toLowerCase();
      return prisma.proof.count({
        where: {
          ...baseWhere,
          OR: [
            { title: { contains: searchTerm, mode: 'insensitive' } },
            { description: { contains: searchTerm, mode: 'insensitive' } },
          ],
        },
      });
    }

    return prisma.proof.count({ where: baseWhere });
  }

  private getBaseProofWhere(
    departmentId: string | null,
    role: UserRole,
    filters: SearchFilters
  ): Prisma.ProofWhereInput {
    const where: Prisma.ProofWhereInput = {};

    if (role !== UserRole.OrgAdmin && departmentId) {
      where.folder = { departmentId };
    }

    if (filters.departmentId) {
      where.folder = { ...where.folder as Prisma.FolderWhereInput, departmentId: filters.departmentId };
    }

    if (filters.status) {
      where.status = filters.status;
    }

    if (filters.createdById) {
      where.createdById = filters.createdById;
    }

    if (filters.dateFrom || filters.dateTo) {
      where.createdAt = {};
      if (filters.dateFrom) {
        where.createdAt.gte = filters.dateFrom;
      }
      if (filters.dateTo) {
        where.createdAt.lte = filters.dateTo;
      }
    }

    return where;
  }

  private getBaseFolderWhere(
    departmentId: string | null,
    role: UserRole
  ): Prisma.FolderWhereInput {
    if (role === UserRole.OrgAdmin) {
      return {};
    }

    return {
      departmentId: departmentId || undefined,
    };
  }
}

export const searchService = new SearchService();
