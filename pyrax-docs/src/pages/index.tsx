import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import styles from './index.module.css';

const categories = [
  {
    title: 'General Documentation',
    description: 'Learn about PYRAX, tokenomics, mining, staking, and governance.',
    icon: '📚',
    link: '/docs/general/getting-started',
    items: ['What is PYRAX?', 'Tokenomics', 'Mining Guide', 'Staking', 'DAO Governance'],
  },
  {
    title: 'Developer Documentation',
    description: 'Build on PYRAX with our comprehensive developer guides and API references.',
    icon: '⚡',
    link: '/docs/developers/overview',
    items: ['Smart Contracts (Solidity)', 'WASM/Rust Contracts', 'APIs & SDKs', 'Crucible AI Platform'],
  },
];

const quickLinks = [
  { title: 'Getting Started', link: '/docs/general/getting-started', icon: '🚀' },
  { title: 'Quick Start (Dev)', link: '/docs/developers/quickstart', icon: '💻' },
  { title: 'Mining Setup', link: '/docs/general/mining-setup', icon: '⛏️' },
  { title: 'API Reference', link: '/docs/developers/api-reference', icon: '📡' },
  { title: 'WASM/Rust', link: '/docs/developers/wasm-overview', icon: '🦀' },
  { title: 'Crucible AI', link: '/docs/developers/crucible-overview', icon: '🤖' },
];

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={styles.hero}>
      <div className={styles.heroInner}>
        <div className={styles.heroLogo}>
          <img src="/img/pyrax-logo.svg" alt="PYRAX" className={styles.logo} />
        </div>
        <h1 className={styles.heroTitle}>PYRAX Documentation</h1>
        <p className={styles.heroSubtitle}>{siteConfig.tagline}</p>
        <div className={styles.heroButtons}>
          <Link className={styles.primaryButton} to="/docs/general/getting-started">
            Get Started
          </Link>
          <Link className={styles.secondaryButton} to="/docs/developers/overview">
            Developer Docs
          </Link>
        </div>
      </div>
      <div className={styles.heroGlow} />
    </header>
  );
}

function CategoryCard({title, description, icon, link, items}) {
  return (
    <Link to={link} className={styles.categoryCard}>
      <div className={styles.categoryIcon}>{icon}</div>
      <h3 className={styles.categoryTitle}>{title}</h3>
      <p className={styles.categoryDescription}>{description}</p>
      <ul className={styles.categoryItems}>
        {items.map((item, idx) => (
          <li key={idx}>{item}</li>
        ))}
      </ul>
      <span className={styles.categoryLink}>Explore →</span>
    </Link>
  );
}

function QuickLinks() {
  return (
    <section className={styles.quickLinks}>
      <h2 className={styles.sectionTitle}>Quick Links</h2>
      <div className={styles.quickLinksGrid}>
        {quickLinks.map((item, idx) => (
          <Link key={idx} to={item.link} className={styles.quickLinkCard}>
            <span className={styles.quickLinkIcon}>{item.icon}</span>
            <span className={styles.quickLinkTitle}>{item.title}</span>
          </Link>
        ))}
      </div>
    </section>
  );
}

function Categories() {
  return (
    <section className={styles.categories}>
      <h2 className={styles.sectionTitle}>Documentation</h2>
      <div className={styles.categoriesGrid}>
        {categories.map((cat, idx) => (
          <CategoryCard key={idx} {...cat} />
        ))}
      </div>
    </section>
  );
}

function CTA() {
  return (
    <section className={styles.cta}>
      <div className={styles.ctaContent}>
        <h2>Ready to build on PYRAX?</h2>
        <p>Join our community and start building the future of decentralized AI computing.</p>
        <div className={styles.ctaButtons}>
          <Link className={styles.primaryButton} href="https://discord.gg/sS7kaacRwU">
            Join Discord
          </Link>
          <Link className={styles.secondaryButton} href="https://github.com/PYRAX-Chain">
            View GitHub
          </Link>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="Documentation"
      description="Complete documentation for the PYRAX Network - AI-Powered GPU Mining & Decentralized Computing">
      <HomepageHeader />
      <main className={styles.main}>
        <QuickLinks />
        <Categories />
        <CTA />
      </main>
    </Layout>
  );
}
