'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Image from 'next/image';
import {
  UserGroupIcon,
  RocketLaunchIcon,
  CodeBracketIcon,
  ChatBubbleLeftRightIcon,
  EnvelopeIcon,
} from '@heroicons/react/24/outline';

interface SocialLink {
  platform: string;
  url: string;
  icon: string;
}

interface TeamMember {
  name: string;
  role: string;
  bio: string;
  image?: string;
  placeholder?: boolean;
  socials?: SocialLink[];
}

export default function TeamPage() {
  const t = useTranslations('teamPage');

  const foundingTeam: TeamMember[] = [
    {
      name: 'Shawn Wilson',
      role: t('roles.coFounder'),
      bio: t('bios.shawn'),
      image: '/swilson.jpg',
      socials: [
        { platform: 'WhatsApp', url: 'https://wa.me/18258825915', icon: 'whatsapp' },
        { platform: 'Email', url: 'mailto:shawn.wilson@pyrax.org', icon: 'email' },
        { platform: 'Facebook', url: 'https://www.facebook.com/SW037', icon: 'facebook' },
        { platform: 'Twitter', url: 'https://x.com/pyrax_shawn', icon: 'twitter' },
        { platform: 'LinkedIn', url: 'https://www.linkedin.com/in/swilson-pyrax', icon: 'linkedin' },
        { platform: 'Telegram', url: 'https://t.me/R3AP3RW1LLY', icon: 'telegram' },
        { platform: 'Discord', url: 'https://discord.com/users/r3ap3ractual_22545', icon: 'discord' },
      ],
    },
    {
      name: 'Gabriel Mascioli',
      role: t('roles.svp'),
      bio: t('bios.gabriel'),
    },
    {
      name: 'Tekky Natale',
      role: t('roles.cfo'),
      bio: t('bios.tekky'),
    },
  ];

  const coreTeam: TeamMember[] = [
    { name: t('comingSoon'), role: t('roles.coreDev'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.coreDev'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.coreDev'), bio: t('bios.comingSoon'), placeholder: true },
  ];

  const communityTeam: TeamMember[] = [
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
    { name: t('comingSoon'), role: t('roles.communityManager'), bio: t('bios.comingSoon'), placeholder: true },
  ];

  const TeamCard = ({ member, index }: { member: TeamMember; index: number }) => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay: index * 0.1 }}
      className="group relative bg-stone-900/50 border border-stone-800 rounded-2xl overflow-hidden hover:border-pyrax-500/50 transition-all duration-300"
    >
      {/* Image or Placeholder */}
      <div className="relative h-64 bg-gradient-to-br from-stone-800 to-stone-900 overflow-hidden">
        {member.image ? (
          <Image
            src={member.image}
            alt={member.name}
            fill
            className="object-cover object-center group-hover:scale-105 transition-transform duration-500"
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className={`w-24 h-24 rounded-full ${member.placeholder ? 'bg-stone-800' : 'bg-gradient-to-br from-pyrax-500 to-orange-500'} flex items-center justify-center`}>
              <UserGroupIcon className="w-12 h-12 text-stone-500" />
            </div>
          </div>
        )}
        {/* Gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-stone-900 via-transparent to-transparent" />
      </div>

      {/* Content */}
      <div className="p-6">
        <h3 className={`text-xl font-bold mb-1 ${member.placeholder ? 'text-stone-500' : 'text-white'}`}>
          {member.name}
        </h3>
        <p className="text-pyrax-500 font-medium mb-3">{member.role}</p>
        <p className="text-stone-400 text-sm">{member.bio}</p>
        
        {/* Social Links */}
        {member.socials && member.socials.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-4 pt-4 border-t border-stone-800">
            {member.socials.map((social) => (
              <a
                key={social.platform}
                href={social.url}
                target="_blank"
                rel="noopener noreferrer"
                className="w-9 h-9 rounded-lg bg-stone-800 hover:bg-pyrax-500/20 flex items-center justify-center transition-colors group"
                title={social.platform}
              >
                {social.icon === 'whatsapp' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-green-500" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413z"/>
                  </svg>
                )}
                {social.icon === 'email' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-pyrax-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                  </svg>
                )}
                {social.icon === 'facebook' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-blue-500" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/>
                  </svg>
                )}
                {social.icon === 'twitter' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-sky-500" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
                  </svg>
                )}
                {social.icon === 'linkedin' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-blue-600" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
                  </svg>
                )}
                {social.icon === 'telegram' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-sky-400" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/>
                  </svg>
                )}
                {social.icon === 'discord' && (
                  <svg className="w-4 h-4 text-stone-400 group-hover:text-indigo-500" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>
                  </svg>
                )}
              </a>
            ))}
          </div>
        )}
      </div>
    </motion.div>
  );

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-pyrax-500/5 blur-[128px]" />
        <div className="absolute top-1/3 right-1/4 w-[400px] h-[400px] rounded-full bg-orange-500/5 blur-[100px]" />

        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
              {t('hero.title')}
            </h1>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto">
              {t('hero.subtitle')}
            </p>
          </motion.div>
        </div>
      </section>

      {/* Founding Team */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-pyrax-500/10 border border-pyrax-500/20 mb-6">
              <RocketLaunchIcon className="w-5 h-5 text-pyrax-500" />
              <span className="text-pyrax-400 font-medium">{t('sections.founding.badge')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-4">{t('sections.founding.title')}</h2>
            <p className="text-stone-400 max-w-2xl mx-auto">{t('sections.founding.subtitle')}</p>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-8">
            {foundingTeam.map((member, index) => (
              <TeamCard key={member.name} member={member} index={index} />
            ))}
          </div>
        </div>
      </section>

      {/* Core Team */}
      <section className="py-20 bg-stone-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-blue-500/10 border border-blue-500/20 mb-6">
              <CodeBracketIcon className="w-5 h-5 text-blue-500" />
              <span className="text-blue-400 font-medium">{t('sections.core.badge')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-4">{t('sections.core.title')}</h2>
            <p className="text-stone-400 max-w-2xl mx-auto">{t('sections.core.subtitle')}</p>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-8">
            {coreTeam.map((member, index) => (
              <TeamCard key={`core-${index}`} member={member} index={index} />
            ))}
          </div>
        </div>
      </section>

      {/* Community Team */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-purple-500/10 border border-purple-500/20 mb-6">
              <ChatBubbleLeftRightIcon className="w-5 h-5 text-purple-500" />
              <span className="text-purple-400 font-medium">{t('sections.community.badge')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-4">{t('sections.community.title')}</h2>
            <p className="text-stone-400 max-w-2xl mx-auto">{t('sections.community.subtitle')}</p>
          </motion.div>

          <div className="grid md:grid-cols-3 lg:grid-cols-3 gap-8">
            {communityTeam.map((member, index) => (
              <TeamCard key={`community-${index}`} member={member} index={index} />
            ))}
          </div>
        </div>
      </section>

      {/* Join Us CTA */}
      <section className="py-20 bg-gradient-to-b from-stone-900/50 to-stone-950">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <h2 className="text-4xl font-bold text-white mb-6">{t('cta.title')}</h2>
            <p className="text-xl text-stone-400 mb-10 max-w-2xl mx-auto">
              {t('cta.subtitle')}
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <a
                href="https://discord.gg/z9kjrE9q"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-8 py-4 bg-gradient-to-r from-pyrax-500 to-orange-500 text-white font-semibold rounded-xl hover:shadow-lg hover:shadow-pyrax-500/25 transition-all"
              >
                <img src="/icons/discord.svg" alt="Discord" className="w-5 h-5" />
                {t('cta.joinDiscord')}
              </a>
              <a
                href="mailto:team@pyrax.org"
                className="inline-flex items-center gap-2 px-8 py-4 bg-stone-800 text-white font-semibold rounded-xl hover:bg-stone-700 transition-all border border-stone-700"
              >
                <EnvelopeIcon className="w-5 h-5" />
                {t('cta.contactUs')}
              </a>
            </div>
          </motion.div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
