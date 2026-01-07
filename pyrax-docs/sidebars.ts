import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  generalSidebar: [
    {
      type: 'category',
      label: 'Introduction',
      collapsed: false,
      items: [
        'general/getting-started',
        'general/what-is-pyrax',
        'general/why-pyrax',
      ],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      items: [
        'general/tokenomics',
        'general/dual-stream-mining',
        'general/gpu-mining',
        'general/ai-compute',
        'general/staking',
      ],
    },
    {
      type: 'category',
      label: 'Getting PYRAX',
      items: [
        'general/buy-pyrax',
        'general/presale',
        'general/exchanges',
        'general/wallets',
      ],
    },
    {
      type: 'category',
      label: 'Mining Guide',
      items: [
        'general/mining-overview',
        'general/mining-requirements',
        'general/mining-setup',
        'general/mining-pools',
        'general/mining-rewards',
      ],
    },
    {
      type: 'category',
      label: 'Governance',
      items: [
        'general/dao-overview',
        'general/voting',
        'general/proposals',
      ],
    },
    {
      type: 'category',
      label: 'Security',
      items: [
        'general/security-best-practices',
        'general/scam-prevention',
      ],
    },
    {
      type: 'category',
      label: 'FAQ',
      items: [
        'general/faq',
      ],
    },
  ],
  developersSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'developers/overview',
        'developers/quickstart',
        'developers/architecture',
      ],
    },
    {
      type: 'category',
      label: 'Network',
      items: [
        'developers/network-overview',
        'developers/consensus',
        'developers/block-structure',
        'developers/transaction-format',
      ],
    },
    {
      type: 'category',
      label: 'Smart Contracts (Solidity)',
      items: [
        'developers/smart-contracts-intro',
        'developers/contract-development',
        'developers/contract-deployment',
        'developers/contract-security',
      ],
    },
    {
      type: 'category',
      label: 'WASM/Rust Contracts',
      items: [
        'developers/wasm-overview',
        'developers/rust-setup',
        'developers/rust-contract-development',
        'developers/rust-contract-deployment',
        'developers/wasm-best-practices',
      ],
    },
    {
      type: 'category',
      label: 'APIs & SDKs',
      items: [
        'developers/api-reference',
        'developers/rpc-endpoints',
        'developers/javascript-sdk',
        'developers/python-sdk',
      ],
    },
    {
      type: 'category',
      label: 'Node Operations',
      items: [
        'developers/running-a-node',
        'developers/node-configuration',
        'developers/node-maintenance',
      ],
    },
    {
      type: 'category',
      label: 'Crucible (AI Platform)',
      items: [
        'developers/crucible-overview',
        'developers/crucible-integration',
        'developers/job-submission',
        'developers/gpu-provider',
      ],
    },
    {
      type: 'category',
      label: 'Tools & Resources',
      items: [
        'developers/block-explorer',
        'developers/testnet',
        'developers/faucet',
      ],
    },
  ],
};

export default sidebars;
