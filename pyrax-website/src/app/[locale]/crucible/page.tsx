'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import {
  BoltIcon,
  CpuChipIcon,
  ServerIcon,
  ShieldCheckIcon,
  ClockIcon,
  CurrencyDollarIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  PlayIcon,
  QueueListIcon,
  ChartBarIcon,
} from '@heroicons/react/24/outline';

export default function CruciblePage() {
  const t = useTranslations('cruciblePage');

  const jobTypes = [
    { name: t('jobTypes.inference'), icon: BoltIcon, color: 'bg-green-500/20 text-green-400' },
    { name: t('jobTypes.training'), icon: CpuChipIcon, color: 'bg-blue-500/20 text-blue-400' },
    { name: t('jobTypes.fineTuning'), icon: ArrowPathIcon, color: 'bg-purple-500/20 text-purple-400' },
    { name: t('jobTypes.embedding'), icon: ChartBarIcon, color: 'bg-pyrax-500/20 text-pyrax-400' },
  ];

  const workflow = [
    {
      step: '1',
      title: t('workflow.step1.title'),
      description: t('workflow.step1.description'),
      details: t('workflow.step1.details'),
      icon: QueueListIcon,
      color: 'from-blue-500 to-cyan-500',
    },
    {
      step: '2',
      title: t('workflow.step2.title'),
      description: t('workflow.step2.description'),
      details: t('workflow.step2.details'),
      icon: ServerIcon,
      color: 'from-purple-500 to-pink-500',
    },
    {
      step: '3',
      title: t('workflow.step3.title'),
      description: t('workflow.step3.description'),
      details: t('workflow.step3.details'),
      icon: PlayIcon,
      color: 'from-green-500 to-emerald-500',
    },
    {
      step: '4',
      title: t('workflow.step4.title'),
      description: t('workflow.step4.description'),
      details: t('workflow.step4.details'),
      icon: ShieldCheckIcon,
      color: 'from-pyrax-500 to-orange-500',
    },
    {
      step: '5',
      title: t('workflow.step5.title'),
      description: t('workflow.step5.description'),
      details: t('workflow.step5.details'),
      icon: CurrencyDollarIcon,
      color: 'from-yellow-500 to-orange-500',
    },
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-blue-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/3 left-1/4 w-[500px] h-[500px] rounded-full bg-blue-500/10 blur-[128px]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5 }}
            >
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-blue-500/20 border border-blue-500/30 mb-6">
                <BoltIcon className="h-5 w-5 text-blue-400" />
                <span className="text-blue-400 font-medium">{t('hero.badge')}</span>
              </div>
              <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
                {t('hero.title')}
              </h1>
              <p className="text-xl text-stone-400 mb-8">
                {t('hero.subtitle')}
              </p>
              
              <div className="flex flex-wrap gap-3 mb-8">
                {jobTypes.map((type) => (
                  <span key={type.name} className={`inline-flex items-center gap-2 px-4 py-2 rounded-lg ${type.color}`}>
                    <type.icon className="h-4 w-4" />
                    {type.name}
                  </span>
                ))}
              </div>
            </motion.div>
            
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="relative"
            >
              <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-3 h-3 rounded-full bg-red-500" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500" />
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                  <span className="ml-2 text-stone-500 text-sm">crucible.pyrax.org</span>
                </div>
                <div className="space-y-3 font-mono text-sm">
                  <div className="text-stone-500"># Submit an inference job</div>
                  <div className="text-blue-400">pyrax crucible submit \</div>
                  <div className="text-stone-300 pl-4">--model "llama-3-70b@1.0.0" \</div>
                  <div className="text-stone-300 pl-4">--type "inference" \</div>
                  <div className="text-stone-300 pl-4">--input "Explain blockchain" \</div>
                  <div className="text-stone-300 pl-4">--max-tokens 500 \</div>
                  <div className="text-stone-300 pl-4">--budget "10 PYRAX"</div>
                  <div className="mt-4 text-yellow-400">⏳ Job queued: job_0x8f2a...</div>
                  <div className="text-green-400">✓ Matched to provider: gpu_0x3b1c...</div>
                  <div className="text-green-400">✓ Execution complete (2.3s)</div>
                  <div className="text-stone-500">Cost: 0.023 PYRAX</div>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* GPU Mining Pool Integration */}
      <section className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('mining.title')}</h2>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto">{t('mining.subtitle')}</p>
          </div>
          
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <h3 className="text-2xl font-bold text-white mb-6">{t('mining.howItWorks')}</h3>
              <div className="space-y-4">
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-lg bg-pyrax-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                    <span className="text-pyrax-400 font-bold">1</span>
                  </div>
                  <div>
                    <h4 className="text-lg font-semibold text-white mb-1">{t('mining.step1.title')}</h4>
                    <p className="text-stone-400">{t('mining.step1.description')}</p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-lg bg-pyrax-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                    <span className="text-pyrax-400 font-bold">2</span>
                  </div>
                  <div>
                    <h4 className="text-lg font-semibold text-white mb-1">{t('mining.step2.title')}</h4>
                    <p className="text-stone-400">{t('mining.step2.description')}</p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-lg bg-pyrax-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                    <span className="text-pyrax-400 font-bold">3</span>
                  </div>
                  <div>
                    <h4 className="text-lg font-semibold text-white mb-1">{t('mining.step3.title')}</h4>
                    <p className="text-stone-400">{t('mining.step3.description')}</p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 rounded-lg bg-pyrax-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                    <span className="text-pyrax-400 font-bold">4</span>
                  </div>
                  <div>
                    <h4 className="text-lg font-semibold text-white mb-1">{t('mining.step4.title')}</h4>
                    <p className="text-stone-400">{t('mining.step4.description')}</p>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <h4 className="text-lg font-bold text-white mb-4">{t('mining.diagram.title')}</h4>
              <div className="space-y-4">
                <div className="flex items-center justify-between p-4 bg-blue-500/10 border border-blue-500/30 rounded-xl">
                  <div className="flex items-center gap-3">
                    <QueueListIcon className="h-6 w-6 text-blue-400" />
                    <span className="text-white font-medium">{t('mining.diagram.jobQueue')}</span>
                  </div>
                  <span className="text-blue-400">→</span>
                </div>
                <div className="flex items-center justify-between p-4 bg-purple-500/10 border border-purple-500/30 rounded-xl">
                  <div className="flex items-center gap-3">
                    <ServerIcon className="h-6 w-6 text-purple-400" />
                    <span className="text-white font-medium">{t('mining.diagram.matchmaker')}</span>
                  </div>
                  <span className="text-purple-400">→</span>
                </div>
                <div className="flex items-center justify-between p-4 bg-pyrax-500/10 border border-pyrax-500/30 rounded-xl">
                  <div className="flex items-center gap-3">
                    <CpuChipIcon className="h-6 w-6 text-pyrax-400" />
                    <span className="text-white font-medium">{t('mining.diagram.gpuPool')}</span>
                  </div>
                  <span className="text-pyrax-400">→</span>
                </div>
                <div className="flex items-center justify-between p-4 bg-green-500/10 border border-green-500/30 rounded-xl">
                  <div className="flex items-center gap-3">
                    <CheckCircleIcon className="h-6 w-6 text-green-400" />
                    <span className="text-white font-medium">{t('mining.diagram.verification')}</span>
                  </div>
                  <span className="text-green-400">✓</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Job Execution Workflow */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('execution.title')}</h2>
            <p className="text-xl text-stone-400">{t('execution.subtitle')}</p>
          </div>
          
          <div className="grid md:grid-cols-5 gap-4">
            {workflow.map((step, index) => (
              <motion.div
                key={step.step}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="relative"
              >
                <div className="bg-stone-900 border border-stone-800 rounded-2xl p-5 h-full">
                  <div className={`w-12 h-12 rounded-xl bg-gradient-to-br ${step.color} flex items-center justify-center mb-4`}>
                    <step.icon className="h-6 w-6 text-white" />
                  </div>
                  <div className="text-xs text-stone-500 mb-1">Step {step.step}</div>
                  <h3 className="text-lg font-bold text-white mb-2">{step.title}</h3>
                  <p className="text-sm text-stone-400 mb-3">{step.description}</p>
                  <p className="text-xs text-stone-500">{step.details}</p>
                </div>
                {index < workflow.length - 1 && (
                  <div className="hidden md:block absolute top-1/2 -right-2 w-4 h-0.5 bg-stone-700" />
                )}
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Verification Methods */}
      <section className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('verification.title')}</h2>
            <p className="text-xl text-stone-400">{t('verification.subtitle')}</p>
          </div>
          
          <div className="grid md:grid-cols-3 gap-8">
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-green-500/20 flex items-center justify-center mb-4">
                <CheckCircleIcon className="h-6 w-6 text-green-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('verification.consensus.title')}</h3>
              <p className="text-stone-400 mb-4">{t('verification.consensus.description')}</p>
              <div className="text-sm text-stone-500">
                <p>• {t('verification.consensus.detail1')}</p>
                <p>• {t('verification.consensus.detail2')}</p>
              </div>
            </div>
            
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-blue-500/20 flex items-center justify-center mb-4">
                <ShieldCheckIcon className="h-6 w-6 text-blue-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('verification.zkproof.title')}</h3>
              <p className="text-stone-400 mb-4">{t('verification.zkproof.description')}</p>
              <div className="text-sm text-stone-500">
                <p>• {t('verification.zkproof.detail1')}</p>
                <p>• {t('verification.zkproof.detail2')}</p>
              </div>
            </div>
            
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-pyrax-500/20 flex items-center justify-center mb-4">
                <ExclamationTriangleIcon className="h-6 w-6 text-pyrax-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('verification.dispute.title')}</h3>
              <p className="text-stone-400 mb-4">{t('verification.dispute.description')}</p>
              <div className="text-sm text-stone-500">
                <p>• {t('verification.dispute.detail1')}</p>
                <p>• {t('verification.dispute.detail2')}</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
