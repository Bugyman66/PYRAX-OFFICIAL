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

interface TeamMember {
  name: string;
  role: string;
  bio: string;
  image?: string;
  placeholder?: boolean;
}

export default function TeamPage() {
  const t = useTranslations('teamPage');

  const foundingTeam: TeamMember[] = [
    {
      name: 'Shawn Wilson',
      role: t('roles.coFounder'),
      bio: t('bios.shawn'),
      image: 'https://pyrax-assets.nyc3.cdn.digitaloceanspaces.com/team%20images/swilson.jpg',
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
