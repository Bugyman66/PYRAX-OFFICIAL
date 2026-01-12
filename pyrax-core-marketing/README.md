# PYRAX Proofing Hub

Enterprise-grade online proofing, review, and approvals system for PYRAX Blockchain's marketing and brand departments.

**Production URL:** https://marketing.pyrax.org

## Features

- **Proof Review** - PDF, image, and video viewer with annotations
- **Workflow Engine** - Department-controlled approval workflows
- **Comments & Mentions** - Threaded discussions with @mentions
- **Public Portal** - Media kit and public asset library
- **External Submissions** - Public content submission portal
- **Audit Trail** - Complete history of all actions

## Tech Stack

- **Framework:** Next.js 14 (App Router) + TypeScript
- **Database:** PostgreSQL + Prisma ORM
- **Styling:** TailwindCSS v4.1.17 (CLI only)
- **Auth:** Magic-link (passwordless) via Brevo
- **Storage:** Google Shared Drive
- **Email:** Brevo Transactional API

## Getting Started

### Prerequisites

- Node.js 18+
- PostgreSQL database
- Google Cloud project with Drive API enabled
- Brevo account for transactional email

### Installation

```bash
# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Edit .env with your configuration
```

### Environment Variables

```env
# Database
DATABASE_URL="postgresql://user:password@localhost:5432/pyrax_proofing_hub"

# Authentication
NEXTAUTH_SECRET="your-secret-key-min-32-chars"
NEXTAUTH_URL="https://marketing.pyrax.org"

# Google Drive
GOOGLE_DRIVE_CLIENT_ID=""
GOOGLE_DRIVE_CLIENT_SECRET=""
GOOGLE_DRIVE_FOLDER_ID=""

# Brevo
BREVO_API_KEY=""
BREVO_SENDER_EMAIL="noreply@pyrax.org"

# Application
NEXT_PUBLIC_APP_URL="https://marketing.pyrax.org"
INTERNAL_EMAIL_DOMAIN="pyrax.org"
```

### Database Setup

```bash
# Generate Prisma client
npm run db:generate

# Push schema to database
npm run db:push

# Seed with sample data
npm run db:seed
```

### Development

```bash
# Run development server (includes Tailwind watch)
npm run dev
```

Open http://localhost:3000

### Production Build

```bash
npm run build
npm start
```

## User Roles

| Role | Description |
|------|-------------|
| **OrgAdmin** | Full system access |
| **DepartmentHead** | Manage workflows, publish assets |
| **Employee** | Create, review, annotate |
| **PublicSubmitter** | Submit content, view status |

## Routes

### Internal (`/app/*`)
- `/app` - Dashboard
- `/app/folders` - Browse folders
- `/app/proofs/[id]` - Proof viewer
- `/app/department/*` - Department management
- `/app/admin/*` - Admin settings

### Public
- `/media-kit` - Brand guidelines (no auth)
- `/public-library` - Public assets (no auth)
- `/submit` - Content submission (PublicSubmitter)

## Seed Data

The seed script creates:
- 2 Departments (Graphics & Branding, Content Creation)
- 5 Users (@pyrax.org emails)
- 3 Workflow Templates
- 2 Sample Folders

Login with any @pyrax.org email to receive a magic link.

## Documentation

- [Project Overview](./project_overview.md)
- [Build Plan](./build_plan.md)

## License

© 2026 PYRAX Blockchain. All rights reserved.
