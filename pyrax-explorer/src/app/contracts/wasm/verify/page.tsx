'use client'

import { useState, useEffect } from 'react'
import Link from 'next/link'
import {
  CheckCircleIcon,
  ExclamationCircleIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  DocumentTextIcon,
  CommandLineIcon,
  CogIcon,
  ShieldCheckIcon,
  ChevronDownIcon,
  PlusIcon,
  TrashIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline'
import { CheckIcon } from '@heroicons/react/20/solid'

interface FormData {
  contractAddress: string
  contractName: string
  crateVersion: string
  rustVersion: string
  optimizationLevel: string
  wasmTarget: string
  sourceCode: string
  cargoToml: string
  initArguments: string
  license: string
  features: string[]
  dependencies: Array<{ name: string; version: string }>
}

interface FormErrors {
  [key: string]: string
}

interface VerificationOptions {
  rustVersions: string[]
  optimizationLevels: Array<{ value: string; label: string }>
  licenseTypes: Array<{ value: string; label: string }>
  wasmTargets: Array<{ value: string; label: string }>
  commonFeatures: string[]
}

const STEPS = [
  { id: 1, name: 'Contract Details', icon: DocumentTextIcon },
  { id: 2, name: 'Build Settings', icon: CogIcon },
  { id: 3, name: 'Source Code', icon: CommandLineIcon },
  { id: 4, name: 'Verify', icon: ShieldCheckIcon },
]

const initialFormData: FormData = {
  contractAddress: '',
  contractName: '',
  crateVersion: '0.1.0',
  rustVersion: '1.75.0',
  optimizationLevel: 'release',
  wasmTarget: 'wasm32-unknown-unknown',
  sourceCode: '',
  cargoToml: '',
  initArguments: '',
  license: 'mit',
  features: [],
  dependencies: [],
}

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

export default function VerifyWasmContractPage() {
  const [currentStep, setCurrentStep] = useState(1)
  const [formData, setFormData] = useState<FormData>(initialFormData)
  const [errors, setErrors] = useState<FormErrors>({})
  const [options, setOptions] = useState<VerificationOptions | null>(null)
  const [verifying, setVerifying] = useState(false)
  const [verificationResult, setVerificationResult] = useState<{
    success: boolean
    message: string
  } | null>(null)

  // Fetch verification options on mount
  useEffect(() => {
    async function fetchOptions() {
      try {
        const response = await fetch('/api/contracts/wasm/verify')
        const data = await response.json()
        setOptions(data)
      } catch (error) {
        console.error('Failed to fetch options:', error)
      }
    }
    fetchOptions()
  }, [])

  const updateFormData = (field: keyof FormData, value: unknown) => {
    setFormData(prev => ({ ...prev, [field]: value }))
    // Clear error when field is updated
    if (errors[field]) {
      setErrors(prev => {
        const newErrors = { ...prev }
        delete newErrors[field]
        return newErrors
      })
    }
  }

  const toggleFeature = (feature: string) => {
    setFormData(prev => ({
      ...prev,
      features: prev.features.includes(feature)
        ? prev.features.filter(f => f !== feature)
        : [...prev.features, feature],
    }))
  }

  const addDependency = () => {
    setFormData(prev => ({
      ...prev,
      dependencies: [...prev.dependencies, { name: '', version: '' }],
    }))
  }

  const removeDependency = (index: number) => {
    setFormData(prev => ({
      ...prev,
      dependencies: prev.dependencies.filter((_, i) => i !== index),
    }))
  }

  const updateDependency = (index: number, field: 'name' | 'version', value: string) => {
    setFormData(prev => ({
      ...prev,
      dependencies: prev.dependencies.map((dep, i) =>
        i === index ? { ...dep, [field]: value } : dep
      ),
    }))
  }

  const validateStep = (step: number): boolean => {
    const newErrors: FormErrors = {}

    if (step === 1) {
      // Contract address validation
      if (!formData.contractAddress) {
        newErrors.contractAddress = 'Contract address is required'
      } else if (!/^0x[a-fA-F0-9]{40}$/.test(formData.contractAddress)) {
        newErrors.contractAddress = 'Invalid contract address format (must be 0x + 40 hex characters)'
      }

      // Contract/crate name validation
      if (!formData.contractName || formData.contractName.trim().length === 0) {
        newErrors.contractName = 'Crate name is required'
      } else if (!/^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(formData.contractName)) {
        newErrors.contractName = 'Crate name must use snake_case or kebab-case'
      }

      // Crate version validation
      if (!formData.crateVersion) {
        newErrors.crateVersion = 'Crate version is required'
      } else if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(formData.crateVersion)) {
        newErrors.crateVersion = 'Invalid version format (use semver, e.g., 0.1.0)'
      }
    }

    if (step === 2) {
      // Rust version validation
      if (!formData.rustVersion) {
        newErrors.rustVersion = 'Rust version is required'
      }

      // Dependency validation
      formData.dependencies.forEach((dep, i) => {
        if (!dep.name || dep.name.trim().length === 0) {
          newErrors[`dependency_${i}_name`] = 'Dependency name is required'
        }
        if (!dep.version || !/^\d+\.\d+(\.\d+)?$/.test(dep.version)) {
          newErrors[`dependency_${i}_version`] = 'Invalid version format'
        }
      })
    }

    if (step === 3) {
      // Source code validation
      if (!formData.sourceCode || formData.sourceCode.trim().length === 0) {
        newErrors.sourceCode = 'Source code is required'
      } else if (formData.sourceCode.length < 50) {
        newErrors.sourceCode = 'Source code appears to be too short'
      } else if (!formData.sourceCode.includes('fn ') && !formData.sourceCode.includes('pub fn ')) {
        newErrors.sourceCode = 'Source code must contain function definitions'
      }

      // Cargo.toml validation
      if (!formData.cargoToml || formData.cargoToml.trim().length === 0) {
        newErrors.cargoToml = 'Cargo.toml is required'
      } else if (!formData.cargoToml.includes('[package]')) {
        newErrors.cargoToml = 'Cargo.toml must include [package] section'
      }

      // Init arguments validation (optional)
      if (formData.initArguments && formData.initArguments.trim().length > 0) {
        try {
          JSON.parse(formData.initArguments)
        } catch {
          const cleanArgs = formData.initArguments.startsWith('0x') 
            ? formData.initArguments.slice(2) 
            : formData.initArguments
          if (!/^[a-fA-F0-9]*$/.test(cleanArgs)) {
            newErrors.initArguments = 'Init arguments must be valid JSON or hex-encoded bytes'
          }
        }
      }
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleNext = () => {
    if (validateStep(currentStep)) {
      setCurrentStep(prev => Math.min(prev + 1, 4))
    }
  }

  const handlePrevious = () => {
    setCurrentStep(prev => Math.max(prev - 1, 1))
  }

  const handleSubmit = async () => {
    if (!validateStep(3)) return

    setVerifying(true)
    setVerificationResult(null)

    try {
      const response = await fetch('/api/contracts/wasm/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      })

      const result = await response.json()

      if (result.success) {
        setVerificationResult({
          success: true,
          message: result.message || 'WASM contract verified successfully!',
        })
      } else {
        if (result.errors) {
          setErrors(result.errors)
        }
        setVerificationResult({
          success: false,
          message: result.message || 'Verification failed',
        })
      }
    } catch (error) {
      setVerificationResult({
        success: false,
        message: 'Failed to submit verification request',
      })
    } finally {
      setVerifying(false)
    }
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8 max-w-4xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <Link
          href="/contracts/wasm"
          className="inline-flex items-center gap-2 text-stone-400 hover:text-white mb-4 transition-colors"
        >
          <ArrowLeftIcon className="h-4 w-4" />
          Back to WASM Contracts
        </Link>
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-purple-500/10">
            <ShieldCheckIcon className="h-6 w-6 text-purple-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Verify Rust/WASM Contract</h1>
            <p className="text-sm text-stone-400">
              Verify and publish your WASM smart contract source code
            </p>
          </div>
        </div>
      </div>

      {/* Steps Progress */}
      <nav aria-label="Progress" className="mb-8">
        <ol className="flex items-center">
          {STEPS.map((step, stepIdx) => (
            <li
              key={step.id}
              className={classNames(
                stepIdx !== STEPS.length - 1 ? 'flex-1' : '',
                'relative'
              )}
            >
              <div className="flex items-center">
                <button
                  onClick={() => step.id < currentStep && setCurrentStep(step.id)}
                  disabled={step.id > currentStep}
                  className={classNames(
                    'relative flex h-10 w-10 items-center justify-center rounded-full transition-colors',
                    currentStep > step.id
                      ? 'bg-purple-500 hover:bg-purple-600'
                      : currentStep === step.id
                      ? 'border-2 border-purple-500 bg-stone-900'
                      : 'border-2 border-stone-700 bg-stone-900'
                  )}
                >
                  {currentStep > step.id ? (
                    <CheckIcon className="h-5 w-5 text-white" />
                  ) : (
                    <step.icon
                      className={classNames(
                        'h-5 w-5',
                        currentStep === step.id ? 'text-purple-500' : 'text-stone-500'
                      )}
                    />
                  )}
                </button>
                {stepIdx !== STEPS.length - 1 && (
                  <div
                    className={classNames(
                      'flex-1 h-0.5 mx-2',
                      currentStep > step.id ? 'bg-purple-500' : 'bg-stone-700'
                    )}
                  />
                )}
              </div>
              <span
                className={classNames(
                  'absolute -bottom-6 left-0 text-xs font-medium whitespace-nowrap',
                  currentStep >= step.id ? 'text-purple-400' : 'text-stone-500'
                )}
              >
                {step.name}
              </span>
            </li>
          ))}
        </ol>
      </nav>

      {/* Form Content */}
      <div className="mt-12 bg-stone-900 rounded-xl border border-stone-800 p-6">
        {/* Step 1: Contract Details */}
        {currentStep === 1 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Contract Details</h2>

            {/* Contract Address */}
            <div>
              <label htmlFor="contractAddress" className="block text-sm font-medium text-stone-300 mb-2">
                Contract Address <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                id="contractAddress"
                value={formData.contractAddress}
                onChange={(e) => updateFormData('contractAddress', e.target.value)}
                placeholder="0x..."
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500',
                  errors.contractAddress ? 'border-red-500' : 'border-stone-700'
                )}
              />
              {errors.contractAddress && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.contractAddress}
                </p>
              )}
            </div>

            {/* Crate Name */}
            <div>
              <label htmlFor="contractName" className="block text-sm font-medium text-stone-300 mb-2">
                Crate Name <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                id="contractName"
                value={formData.contractName}
                onChange={(e) => updateFormData('contractName', e.target.value)}
                placeholder="my_contract"
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500',
                  errors.contractName ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                The crate name as defined in Cargo.toml (use snake_case)
              </p>
              {errors.contractName && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.contractName}
                </p>
              )}
            </div>

            {/* Crate Version */}
            <div>
              <label htmlFor="crateVersion" className="block text-sm font-medium text-stone-300 mb-2">
                Crate Version <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                id="crateVersion"
                value={formData.crateVersion}
                onChange={(e) => updateFormData('crateVersion', e.target.value)}
                placeholder="0.1.0"
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500',
                  errors.crateVersion ? 'border-red-500' : 'border-stone-700'
                )}
              />
              {errors.crateVersion && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.crateVersion}
                </p>
              )}
            </div>

            {/* License */}
            <div>
              <label htmlFor="license" className="block text-sm font-medium text-stone-300 mb-2">
                License Type
              </label>
              <div className="relative">
                <select
                  id="license"
                  value={formData.license}
                  onChange={(e) => updateFormData('license', e.target.value)}
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 px-4 py-3 text-white text-sm appearance-none focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  {options?.licenseTypes.map((license) => (
                    <option key={license.value} value={license.value}>
                      {license.label}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
            </div>
          </div>
        )}

        {/* Step 2: Build Settings */}
        {currentStep === 2 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Build Settings</h2>

            {/* Rust Version */}
            <div>
              <label htmlFor="rustVersion" className="block text-sm font-medium text-stone-300 mb-2">
                Rust Version <span className="text-red-400">*</span>
              </label>
              <div className="relative">
                <select
                  id="rustVersion"
                  value={formData.rustVersion}
                  onChange={(e) => updateFormData('rustVersion', e.target.value)}
                  className={classNames(
                    'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm appearance-none',
                    'focus:outline-none focus:ring-2 focus:ring-purple-500',
                    errors.rustVersion ? 'border-red-500' : 'border-stone-700'
                  )}
                >
                  {options?.rustVersions.map((version) => (
                    <option key={version} value={version}>
                      {version}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
              {errors.rustVersion && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.rustVersion}
                </p>
              )}
            </div>

            {/* WASM Target */}
            <div>
              <label htmlFor="wasmTarget" className="block text-sm font-medium text-stone-300 mb-2">
                WASM Target
              </label>
              <div className="relative">
                <select
                  id="wasmTarget"
                  value={formData.wasmTarget}
                  onChange={(e) => updateFormData('wasmTarget', e.target.value)}
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 px-4 py-3 text-white text-sm appearance-none focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  {options?.wasmTargets.map((target) => (
                    <option key={target.value} value={target.value}>
                      {target.label}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
            </div>

            {/* Optimization Level */}
            <div>
              <label htmlFor="optimizationLevel" className="block text-sm font-medium text-stone-300 mb-2">
                Optimization Level
              </label>
              <div className="relative">
                <select
                  id="optimizationLevel"
                  value={formData.optimizationLevel}
                  onChange={(e) => updateFormData('optimizationLevel', e.target.value)}
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 px-4 py-3 text-white text-sm appearance-none focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  {options?.optimizationLevels.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
            </div>

            {/* Cargo Features */}
            <div>
              <label className="block text-sm font-medium text-stone-300 mb-2">
                Cargo Features
              </label>
              <div className="flex flex-wrap gap-2">
                {options?.commonFeatures.map((feature) => (
                  <button
                    key={feature}
                    type="button"
                    onClick={() => toggleFeature(feature)}
                    className={classNames(
                      'px-3 py-1.5 rounded-lg text-sm font-medium transition-colors',
                      formData.features.includes(feature)
                        ? 'bg-purple-500 text-white'
                        : 'bg-stone-800 text-stone-400 hover:bg-stone-700 hover:text-white border border-stone-700'
                    )}
                  >
                    {feature}
                  </button>
                ))}
              </div>
              <p className="mt-2 text-xs text-stone-500">
                Select the features enabled during compilation
              </p>
            </div>

            {/* Additional Dependencies */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-stone-300">
                  Additional Dependencies (Optional)
                </label>
                <button
                  type="button"
                  onClick={addDependency}
                  className="inline-flex items-center gap-1 text-sm text-purple-400 hover:text-purple-300"
                >
                  <PlusIcon className="h-4 w-4" />
                  Add Dependency
                </button>
              </div>
              
              {formData.dependencies.length > 0 && (
                <div className="space-y-3">
                  {formData.dependencies.map((dep, index) => (
                    <div key={index} className="flex items-start gap-3 p-3 rounded-lg bg-stone-800/50 border border-stone-700">
                      <div className="flex-1 grid grid-cols-2 gap-2">
                        <input
                          type="text"
                          value={dep.name}
                          onChange={(e) => updateDependency(index, 'name', e.target.value)}
                          placeholder="crate_name"
                          className={classNames(
                            'rounded-lg bg-stone-800 border px-3 py-2 text-white text-sm',
                            'focus:outline-none focus:ring-2 focus:ring-purple-500',
                            errors[`dependency_${index}_name`] ? 'border-red-500' : 'border-stone-700'
                          )}
                        />
                        <input
                          type="text"
                          value={dep.version}
                          onChange={(e) => updateDependency(index, 'version', e.target.value)}
                          placeholder="1.0.0"
                          className={classNames(
                            'rounded-lg bg-stone-800 border px-3 py-2 text-white text-sm',
                            'focus:outline-none focus:ring-2 focus:ring-purple-500',
                            errors[`dependency_${index}_version`] ? 'border-red-500' : 'border-stone-700'
                          )}
                        />
                      </div>
                      <button
                        type="button"
                        onClick={() => removeDependency(index)}
                        className="p-2 text-stone-500 hover:text-red-400 transition-colors"
                      >
                        <TrashIcon className="h-5 w-5" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
              
              {formData.dependencies.length === 0 && (
                <p className="text-sm text-stone-500">
                  Add dependencies not in Cargo.toml that were used during compilation
                </p>
              )}
            </div>
          </div>
        )}

        {/* Step 3: Source Code */}
        {currentStep === 3 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Source Code</h2>

            {/* Cargo.toml */}
            <div>
              <label htmlFor="cargoToml" className="block text-sm font-medium text-stone-300 mb-2">
                Cargo.toml <span className="text-red-400">*</span>
              </label>
              <textarea
                id="cargoToml"
                value={formData.cargoToml}
                onChange={(e) => updateFormData('cargoToml', e.target.value)}
                rows={8}
                placeholder={`[package]
name = "my_contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Your dependencies here`}
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500 resize-y',
                  errors.cargoToml ? 'border-red-500' : 'border-stone-700'
                )}
              />
              {errors.cargoToml && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.cargoToml}
                </p>
              )}
            </div>

            {/* Source Code */}
            <div>
              <label htmlFor="sourceCode" className="block text-sm font-medium text-stone-300 mb-2">
                lib.rs Source Code <span className="text-red-400">*</span>
              </label>
              <textarea
                id="sourceCode"
                value={formData.sourceCode}
                onChange={(e) => updateFormData('sourceCode', e.target.value)}
                rows={16}
                placeholder={`#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn init() {
    // Contract initialization
}

#[no_mangle]
pub extern "C" fn call() {
    // Contract entry point
}`}
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500 resize-y',
                  errors.sourceCode ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                Paste the complete lib.rs source code. For multi-file projects, flatten all modules.
              </p>
              {errors.sourceCode && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.sourceCode}
                </p>
              )}
            </div>

            {/* Init Arguments */}
            <div>
              <label htmlFor="initArguments" className="block text-sm font-medium text-stone-300 mb-2">
                Init Arguments (JSON or Hex)
              </label>
              <textarea
                id="initArguments"
                value={formData.initArguments}
                onChange={(e) => updateFormData('initArguments', e.target.value)}
                rows={3}
                placeholder='{"param1": "value1", "param2": 123}'
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-purple-500 resize-y',
                  errors.initArguments ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                Leave empty if the contract init function has no parameters
              </p>
              {errors.initArguments && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.initArguments}
                </p>
              )}
            </div>
          </div>
        )}

        {/* Step 4: Verify */}
        {currentStep === 4 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Review & Verify</h2>

            {/* Summary */}
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Contract Address</p>
                  <p className="text-sm text-white font-mono truncate">{formData.contractAddress}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Crate Name</p>
                  <p className="text-sm text-white">{formData.contractName}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Crate Version</p>
                  <p className="text-sm text-white">{formData.crateVersion}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Rust Version</p>
                  <p className="text-sm text-white">{formData.rustVersion}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">WASM Target</p>
                  <p className="text-sm text-white font-mono text-xs">{formData.wasmTarget}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Optimization</p>
                  <p className="text-sm text-white capitalize">{formData.optimizationLevel}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">License</p>
                  <p className="text-sm text-white uppercase">{formData.license}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Features</p>
                  <p className="text-sm text-white">
                    {formData.features.length > 0 ? formData.features.join(', ') : 'None'}
                  </p>
                </div>
              </div>

              <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Cargo.toml</p>
                <p className="text-sm text-stone-400">
                  {formData.cargoToml.length.toLocaleString()} characters
                </p>
              </div>

              <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Source Code</p>
                <p className="text-sm text-stone-400">
                  {formData.sourceCode.length.toLocaleString()} characters
                </p>
              </div>

              {formData.initArguments && (
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Init Arguments</p>
                  <p className="text-sm text-white font-mono truncate">{formData.initArguments}</p>
                </div>
              )}

              {formData.dependencies.length > 0 && (
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-2">Additional Dependencies</p>
                  {formData.dependencies.map((dep, i) => (
                    <div key={i} className="text-sm">
                      <span className="text-white">{dep.name}</span>
                      <span className="text-stone-500 mx-2">=</span>
                      <span className="text-stone-400">{dep.version}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Verification Result */}
            {verificationResult && (
              <div
                className={classNames(
                  'p-4 rounded-lg border flex items-start gap-3',
                  verificationResult.success
                    ? 'bg-green-500/10 border-green-500/20'
                    : 'bg-red-500/10 border-red-500/20'
                )}
              >
                {verificationResult.success ? (
                  <CheckCircleIcon className="h-5 w-5 text-green-400 shrink-0 mt-0.5" />
                ) : (
                  <ExclamationCircleIcon className="h-5 w-5 text-red-400 shrink-0 mt-0.5" />
                )}
                <div>
                  <p
                    className={classNames(
                      'font-medium',
                      verificationResult.success ? 'text-green-400' : 'text-red-400'
                    )}
                  >
                    {verificationResult.success ? 'Verification Successful!' : 'Verification Failed'}
                  </p>
                  <p className="text-sm text-stone-400 mt-1">{verificationResult.message}</p>
                  {verificationResult.success && (
                    <Link
                      href={`/address/${formData.contractAddress}`}
                      className="inline-flex items-center gap-1 text-sm text-purple-400 hover:text-purple-300 mt-2"
                    >
                      View Contract
                      <ArrowRightIcon className="h-4 w-4" />
                    </Link>
                  )}
                </div>
              </div>
            )}

            {/* Submit Button */}
            {!verificationResult?.success && (
              <button
                type="button"
                onClick={handleSubmit}
                disabled={verifying}
                className="w-full flex items-center justify-center gap-2 rounded-lg bg-purple-500 px-4 py-3 text-sm font-semibold text-white hover:bg-purple-600 disabled:opacity-50 transition-colors"
              >
                {verifying ? (
                  <>
                    <ArrowPathIcon className="h-5 w-5 animate-spin" />
                    Verifying...
                  </>
                ) : (
                  <>
                    <ShieldCheckIcon className="h-5 w-5" />
                    Verify WASM Contract
                  </>
                )}
              </button>
            )}
          </div>
        )}

        {/* Navigation Buttons */}
        <div className="flex justify-between mt-8 pt-6 border-t border-stone-800">
          <button
            type="button"
            onClick={handlePrevious}
            disabled={currentStep === 1}
            className="inline-flex items-center gap-2 rounded-lg bg-stone-800 px-4 py-2 text-sm font-medium text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <ArrowLeftIcon className="h-4 w-4" />
            Previous
          </button>

          {currentStep < 4 && (
            <button
              type="button"
              onClick={handleNext}
              className="inline-flex items-center gap-2 rounded-lg bg-purple-500 px-4 py-2 text-sm font-semibold text-white hover:bg-purple-600 transition-colors"
            >
              Next
              <ArrowRightIcon className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
