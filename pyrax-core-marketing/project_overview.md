# PYRAX Proofing Hub - Project Overview

## Executive Summary

**PYRAX Proofing Hub** is an enterprise-grade online proofing, review, and approvals system designed for PYRAX Blockchain's marketing and brand departments. It provides department-controlled workflows for managing creative assets from creation through approval and publication.

**Production Domain:** `marketing.pyrax.org`

## Project Identity

- **Name:** PYRAX Proofing Hub
- **Internal Codename:** PYRAX Core Marketing
- **Version:** 1.0.0
- **Status:** In Development

## Business Objectives

1. Streamline brand asset review and approval workflows
2. Ensure brand consistency across all marketing materials
3. Provide controlled public access to approved brand assets
4. Enable external parties to submit content for review
5. Maintain complete audit trail for compliance

## Departments Served

### 1. Graphics & Branding
- Logo assets, brand guidelines, visual identity
- Design reviews, color proofs, print materials
- Brand compliance verification

### 2. Content Creation
- Marketing copy, blog posts, social media content
- Video content review with timecode annotations
- Campaign materials and promotional content

## User Types & Roles

| Role | Description | Access Level |
|------|-------------|--------------|
| **OrgAdmin** | Global system administrator | Full access: users, integrations, all departments |
| **DepartmentHead** | Department manager | Workflow templates, approvals, publishing for their department |
| **Employee** | Internal team member | Create, review, annotate within assigned workflows |
| **PublicSubmitter** | External registered user | Submit content, view own submission status |
| **PublicAnonymous** | No login required | View/download public media kit and library |

## Core Features

### 1. Asset Management
- **Folders:** Organize proofs by project, campaign, or category
- **Proofs:** Individual assets with version history
- **Versions:** Track iterations with source files and preview renditions

### 2. Proof Viewer
- **PDF Viewer:** Full pdfjs integration with page navigation
- **Image Viewer:** Support for PNG, JPG, WEBP with zoom/pan
- **Video Viewer:** MP4/WebM playback with timecode markers
- **Annotation Tools:** Pin, rectangle, arrow, freehand drawing
- **Comments:** Threaded discussions tied to annotations or general

### 3. Workflow Engine
- **Templates:** Department-specific workflow configurations
- **Steps:** Sequential approval stages with role assignments
- **Decisions:** Approve / Needs Changes / Reject
- **Rules:** Configurable final approval (single, unanimous, any-one)
- **Audit Trail:** Complete history of all actions

### 4. Dashboards
- "Waiting on Me" - Items requiring user action
- "Overdue" - Past due date items
- "Needs Changes" - Items requiring revision
- "Approved" - Successfully approved items
- Search and filtering across all views

### 5. Public Portal
- **/media-kit:** Brand guidelines and approved assets
- **/public-library:** Downloadable assets for external use
- Permanent links with revocation capability

### 6. Public Submissions
- External content submission portal
- Automatic routing to intake workflow
- Submitter status tracking (without internal details)

### 7. Notifications
- Email notifications via Brevo transactional email
- Triggers: assignment, mention, step change, decisions, approvals

## Technical Architecture

### Stack
| Component | Technology |
|-----------|------------|
| Frontend | Next.js 14 (App Router) + TypeScript |
| Styling | TailwindCSS v4.1.17 (CLI only) |
| Database | PostgreSQL + Prisma ORM |
| Auth | Magic-link (passwordless) via Brevo |
| File Storage | Google Shared Drive (org-level) |
| PDF Rendering | pdfjs-dist |
| Annotations | react-konva |
| Email | Brevo Transactional API |
| Background Jobs | DB-backed queue + Node worker |

### File Type Support

| Type | Preview | Storage | Annotation |
|------|---------|---------|------------|
| PDF | Yes (pdfjs) | Drive | Yes |
| PNG/JPG/WEBP | Yes (native) | Drive | Yes |
| MP4/WebM | Yes (native) | Drive | Timecode markers |
| AI/PSD/etc | Via rendition | Drive | On rendition only |

### Proof Rendition Model
For non-previewable source files (AI, PSD, INDD, etc.):
1. Source file stored in Drive
2. Optional preview rendition (PDF/PNG) uploaded separately
3. Annotations made on rendition, linked to source

## Security & Permissions

### Authentication
- **Internal Users:** Email must end with `@pyrax.org` (server-enforced)
- **External Users:** Standard signup for PublicSubmitter role
- **Magic Link:** Passwordless authentication via email

### Authorization Matrix

| Action | OrgAdmin | DeptHead | Employee | PublicSubmitter | Anonymous |
|--------|----------|----------|----------|-----------------|-----------|
| Manage Users | ✓ | - | - | - | - |
| Manage Integrations | ✓ | - | - | - | - |
| Create Workflow Templates | ✓ | Own Dept | - | - | - |
| Publish Assets | ✓ | Own Dept | - | - | - |
| Create Proofs | ✓ | ✓ | ✓ | - | - |
| Comment/Annotate | ✓ | ✓ | Workflow | - | - |
| Make Decisions | ✓ | ✓ | Workflow | - | - |
| Submit Content | - | - | - | ✓ | - |
| View Own Submissions | - | - | - | ✓ | - |
| Access Media Kit | ✓ | ✓ | ✓ | ✓ | ✓ |
| Download Public Assets | ✓ | ✓ | ✓ | ✓ | ✓ |

## Data Model Overview

```
User
├── Department (optional)
├── Role (OrgAdmin, DepartmentHead, Employee, PublicSubmitter)
└── isInternal (derived from @pyrax.org email)

Department
├── Folders
├── WorkflowTemplates
└── Users (DepartmentHead, Employees)

Folder
└── Proofs

Proof
├── ProofVersions (source + renditions)
├── WorkflowInstance
├── Comments
└── PublicAsset (if published)

ProofVersion
├── Annotations
└── Comments (version-specific)

WorkflowInstance
├── WorkflowTemplate
├── Decisions
└── AuditEvents

ExternalSubmission
├── User (PublicSubmitter)
└── Proof (hidden internal proof)
```

## Integration Points

### Google Drive
- Org-level service account or OAuth refresh token
- All file operations server-side only
- Encrypted token storage
- Folder structure mirrors app hierarchy

### Brevo (Email)
- Transactional email for magic links
- Notification emails for workflow events
- Template-based email content

## URL Structure

### Internal Routes (`/app/*`)
- `/app` - Dashboard
- `/app/folders/[id]` - Folder view
- `/app/proofs/[id]` - Proof viewer
- `/app/department` - Department management
- `/app/admin` - Organization admin

### Public Routes
- `/media-kit` - Brand guidelines (no auth)
- `/public-library` - Public assets (no auth)
- `/submit` - Content submission (PublicSubmitter auth)
- `/me/submissions` - Submission status (PublicSubmitter auth)

## Environment Requirements

```env
# Database
DATABASE_URL=postgresql://...

# Auth
NEXTAUTH_SECRET=...
NEXTAUTH_URL=https://marketing.pyrax.org

# Google Drive
GOOGLE_DRIVE_CLIENT_ID=...
GOOGLE_DRIVE_CLIENT_SECRET=...
GOOGLE_DRIVE_REFRESH_TOKEN=... (encrypted)
GOOGLE_DRIVE_FOLDER_ID=...

# Brevo
BREVO_API_KEY=...
BREVO_SENDER_EMAIL=noreply@pyrax.org
BREVO_SENDER_NAME=PYRAX Proofing Hub

# App
NEXT_PUBLIC_APP_URL=https://marketing.pyrax.org
INTERNAL_EMAIL_DOMAIN=pyrax.org
```

## Success Metrics

1. **Adoption:** 100% of marketing assets flow through system
2. **Efficiency:** 50% reduction in approval cycle time
3. **Compliance:** 100% audit trail coverage
4. **Accessibility:** Public assets available 24/7

## Document References

- `build_plan.md` - Detailed implementation plan
- `README.md` - Setup and deployment guide
- `CHANGELOG.md` - Version history

---

*Last Updated: January 10, 2026*
*Document Version: 1.0*
