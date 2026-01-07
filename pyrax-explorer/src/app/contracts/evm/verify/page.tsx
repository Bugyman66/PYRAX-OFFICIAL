'use client'

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import {
  CheckCircleIcon,
  ExclamationCircleIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  DocumentTextIcon,
  CodeBracketIcon,
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
  compilerVersion: string
  optimization: boolean
  optimizationRuns: number
  evmVersion: string
  sourceCode: string
  constructorArguments: string
  license: string
  libraries: Array<{ name: string; address: string }>
}

interface FormErrors {
  [key: string]: string
}

interface VerificationOptions {
  compilerVersions: string[]
  licenseTypes: Array<{ value: string; label: string }>
  evmVersions: Array<{ value: string; label: string }>
}

const STEPS = [
  { id: 1, name: 'Contract Details', icon: DocumentTextIcon },
  { id: 2, name: 'Compiler Settings', icon: CogIcon },
  { id: 3, name: 'Source Code', icon: CodeBracketIcon },
  { id: 4, name: 'Verify', icon: ShieldCheckIcon },
]

const initialFormData: FormData = {
  contractAddress: '',
  contractName: '',
  compilerVersion: '0.8.20',
  optimization: true,
  optimizationRuns: 200,
  evmVersion: 'default',
  sourceCode: '',
  constructorArguments: '',
  license: 'mit',
  libraries: [],
}

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

export default function VerifyContractPage() {
  const router = useRouter()
  const [currentStep, setCurrentStep] = useState(1)
  const [formData, setFormData] = useState<FormData>(initialFormData)
  const [errors, setErrors] = useState<FormErrors>({})
  const [options, setOptions] = useState<VerificationOptions | null>(null)
  const [loading, setLoading] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [verificationResult, setVerificationResult] = useState<{
    success: boolean
    message: string
  } | null>(null)

  // Fetch verification options on mount
  useEffect(() => {
    async function fetchOptions() {
      try {
        const response = await fetch('/api/contracts/verify')
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

  const addLibrary = () => {
    setFormData(prev => ({
      ...prev,
      libraries: [...prev.libraries, { name: '', address: '' }],
    }))
  }

  const removeLibrary = (index: number) => {
    setFormData(prev => ({
      ...prev,
      libraries: prev.libraries.filter((_, i) => i !== index),
    }))
  }

  const updateLibrary = (index: number, field: 'name' | 'address', value: string) => {
    setFormData(prev => ({
      ...prev,
      libraries: prev.libraries.map((lib, i) =>
        i === index ? { ...lib, [field]: value } : lib
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

      // Contract name validation
      if (!formData.contractName || formData.contractName.trim().length === 0) {
        newErrors.contractName = 'Contract name is required'
      } else if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(formData.contractName)) {
        newErrors.contractName = 'Contract name must start with a letter or underscore and contain only alphanumeric characters'
      }
    }

    if (step === 2) {
      // Compiler version validation
      if (!formData.compilerVersion) {
        newErrors.compilerVersion = 'Compiler version is required'
      }

      // Optimization runs validation
      if (formData.optimization) {
        if (formData.optimizationRuns < 1 || formData.optimizationRuns > 10000000) {
          newErrors.optimizationRuns = 'Optimization runs must be between 1 and 10,000,000'
        }
      }

      // Library validation
      formData.libraries.forEach((lib, i) => {
        if (!lib.name || lib.name.trim().length === 0) {
          newErrors[`library_${i}_name`] = 'Library name is required'
        }
        if (!lib.address || !/^0x[a-fA-F0-9]{40}$/.test(lib.address)) {
          newErrors[`library_${i}_address`] = 'Invalid library address'
        }
      })
    }

    if (step === 3) {
      // Source code validation
      if (!formData.sourceCode || formData.sourceCode.trim().length === 0) {
        newErrors.sourceCode = 'Source code is required'
      } else if (formData.sourceCode.length < 50) {
        newErrors.sourceCode = 'Source code appears to be too short'
      } else if (!formData.sourceCode.includes('pragma solidity') && 
                 !formData.sourceCode.includes('// SPDX-License-Identifier')) {
        newErrors.sourceCode = 'Source code must include pragma solidity directive'
      } else if (!formData.sourceCode.includes('contract ') && 
                 !formData.sourceCode.includes('interface ') && 
                 !formData.sourceCode.includes('library ')) {
        newErrors.sourceCode = 'Source code must define a contract, interface, or library'
      }

      // Constructor arguments validation (optional)
      if (formData.constructorArguments && formData.constructorArguments.trim().length > 0) {
        const cleanArgs = formData.constructorArguments.startsWith('0x') 
          ? formData.constructorArguments.slice(2) 
          : formData.constructorArguments
        if (!/^[a-fA-F0-9]*$/.test(cleanArgs)) {
          newErrors.constructorArguments = 'Constructor arguments must be ABI-encoded hex string'
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
      const response = await fetch('/api/contracts/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      })

      const result = await response.json()

      if (result.success) {
        setVerificationResult({
          success: true,
          message: result.message || 'Contract verified successfully!',
        })
      } else {
        // Handle validation errors from server
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
          href="/contracts/evm"
          className="inline-flex items-center gap-2 text-stone-400 hover:text-white mb-4 transition-colors"
        >
          <ArrowLeftIcon className="h-4 w-4" />
          Back to EVM Contracts
        </Link>
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-pyrax-500/10">
            <ShieldCheckIcon className="h-6 w-6 text-pyrax-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Verify EVM Contract</h1>
            <p className="text-sm text-stone-400">
              Verify and publish your smart contract source code
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
                      ? 'bg-pyrax-500 hover:bg-pyrax-600'
                      : currentStep === step.id
                      ? 'border-2 border-pyrax-500 bg-stone-900'
                      : 'border-2 border-stone-700 bg-stone-900'
                  )}
                >
                  {currentStep > step.id ? (
                    <CheckIcon className="h-5 w-5 text-white" />
                  ) : (
                    <step.icon
                      className={classNames(
                        'h-5 w-5',
                        currentStep === step.id ? 'text-pyrax-500' : 'text-stone-500'
                      )}
                    />
                  )}
                </button>
                {stepIdx !== STEPS.length - 1 && (
                  <div
                    className={classNames(
                      'flex-1 h-0.5 mx-2',
                      currentStep > step.id ? 'bg-pyrax-500' : 'bg-stone-700'
                    )}
                  />
                )}
              </div>
              <span
                className={classNames(
                  'absolute -bottom-6 left-0 text-xs font-medium whitespace-nowrap',
                  currentStep >= step.id ? 'text-pyrax-400' : 'text-stone-500'
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
                  'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
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

            {/* Contract Name */}
            <div>
              <label htmlFor="contractName" className="block text-sm font-medium text-stone-300 mb-2">
                Contract Name <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                id="contractName"
                value={formData.contractName}
                onChange={(e) => updateFormData('contractName', e.target.value)}
                placeholder="MyContract"
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
                  errors.contractName ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                The exact name of the contract as defined in the source code
              </p>
              {errors.contractName && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.contractName}
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
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 px-4 py-3 text-white text-sm appearance-none focus:outline-none focus:ring-2 focus:ring-pyrax-500"
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

        {/* Step 2: Compiler Settings */}
        {currentStep === 2 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Compiler Settings</h2>

            {/* Compiler Version */}
            <div>
              <label htmlFor="compilerVersion" className="block text-sm font-medium text-stone-300 mb-2">
                Compiler Version <span className="text-red-400">*</span>
              </label>
              <div className="relative">
                <select
                  id="compilerVersion"
                  value={formData.compilerVersion}
                  onChange={(e) => updateFormData('compilerVersion', e.target.value)}
                  className={classNames(
                    'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm appearance-none',
                    'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
                    errors.compilerVersion ? 'border-red-500' : 'border-stone-700'
                  )}
                >
                  {options?.compilerVersions.map((version) => (
                    <option key={version} value={version}>
                      v{version}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
              {errors.compilerVersion && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.compilerVersion}
                </p>
              )}
            </div>

            {/* EVM Version */}
            <div>
              <label htmlFor="evmVersion" className="block text-sm font-medium text-stone-300 mb-2">
                EVM Version
              </label>
              <div className="relative">
                <select
                  id="evmVersion"
                  value={formData.evmVersion}
                  onChange={(e) => updateFormData('evmVersion', e.target.value)}
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 px-4 py-3 text-white text-sm appearance-none focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                >
                  {options?.evmVersions.map((evm) => (
                    <option key={evm.value} value={evm.value}>
                      {evm.label}
                    </option>
                  ))}
                </select>
                <ChevronDownIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500 pointer-events-none" />
              </div>
            </div>

            {/* Optimization */}
            <div>
              <div className="flex items-center justify-between">
                <label htmlFor="optimization" className="text-sm font-medium text-stone-300">
                  Enable Optimization
                </label>
                <button
                  type="button"
                  onClick={() => updateFormData('optimization', !formData.optimization)}
                  className={classNames(
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:ring-offset-2 focus:ring-offset-stone-900',
                    formData.optimization ? 'bg-pyrax-500' : 'bg-stone-700'
                  )}
                >
                  <span
                    className={classNames(
                      'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      formData.optimization ? 'translate-x-5' : 'translate-x-0'
                    )}
                  />
                </button>
              </div>
              
              {formData.optimization && (
                <div className="mt-4">
                  <label htmlFor="optimizationRuns" className="block text-sm font-medium text-stone-300 mb-2">
                    Optimization Runs
                  </label>
                  <input
                    type="number"
                    id="optimizationRuns"
                    value={formData.optimizationRuns}
                    onChange={(e) => updateFormData('optimizationRuns', parseInt(e.target.value) || 200)}
                    min={1}
                    max={10000000}
                    className={classNames(
                      'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white text-sm',
                      'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
                      errors.optimizationRuns ? 'border-red-500' : 'border-stone-700'
                    )}
                  />
                  <p className="mt-1 text-xs text-stone-500">
                    Default: 200. Higher values optimize for runtime, lower for deployment.
                  </p>
                  {errors.optimizationRuns && (
                    <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                      <ExclamationCircleIcon className="h-4 w-4" />
                      {errors.optimizationRuns}
                    </p>
                  )}
                </div>
              )}
            </div>

            {/* Libraries */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-stone-300">
                  Contract Libraries (Optional)
                </label>
                <button
                  type="button"
                  onClick={addLibrary}
                  className="inline-flex items-center gap-1 text-sm text-pyrax-400 hover:text-pyrax-300"
                >
                  <PlusIcon className="h-4 w-4" />
                  Add Library
                </button>
              </div>
              
              {formData.libraries.length > 0 && (
                <div className="space-y-3">
                  {formData.libraries.map((lib, index) => (
                    <div key={index} className="flex items-start gap-3 p-3 rounded-lg bg-stone-800/50 border border-stone-700">
                      <div className="flex-1 space-y-2">
                        <input
                          type="text"
                          value={lib.name}
                          onChange={(e) => updateLibrary(index, 'name', e.target.value)}
                          placeholder="Library name (e.g., SafeMath)"
                          className={classNames(
                            'w-full rounded-lg bg-stone-800 border px-3 py-2 text-white text-sm',
                            'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
                            errors[`library_${index}_name`] ? 'border-red-500' : 'border-stone-700'
                          )}
                        />
                        <input
                          type="text"
                          value={lib.address}
                          onChange={(e) => updateLibrary(index, 'address', e.target.value)}
                          placeholder="0x..."
                          className={classNames(
                            'w-full rounded-lg bg-stone-800 border px-3 py-2 text-white font-mono text-sm',
                            'focus:outline-none focus:ring-2 focus:ring-pyrax-500',
                            errors[`library_${index}_address`] ? 'border-red-500' : 'border-stone-700'
                          )}
                        />
                      </div>
                      <button
                        type="button"
                        onClick={() => removeLibrary(index)}
                        className="p-2 text-stone-500 hover:text-red-400 transition-colors"
                      >
                        <TrashIcon className="h-5 w-5" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
              
              {formData.libraries.length === 0 && (
                <p className="text-sm text-stone-500">
                  Add library addresses if your contract uses external libraries
                </p>
              )}
            </div>
          </div>
        )}

        {/* Step 3: Source Code */}
        {currentStep === 3 && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-white mb-4">Source Code</h2>

            {/* Source Code */}
            <div>
              <label htmlFor="sourceCode" className="block text-sm font-medium text-stone-300 mb-2">
                Solidity Source Code <span className="text-red-400">*</span>
              </label>
              <textarea
                id="sourceCode"
                value={formData.sourceCode}
                onChange={(e) => updateFormData('sourceCode', e.target.value)}
                rows={16}
                placeholder={`// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract MyContract {
    // Your contract code here
}`}
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-y',
                  errors.sourceCode ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                Paste the complete, flattened source code including all imports
              </p>
              {errors.sourceCode && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.sourceCode}
                </p>
              )}
            </div>

            {/* Constructor Arguments */}
            <div>
              <label htmlFor="constructorArguments" className="block text-sm font-medium text-stone-300 mb-2">
                Constructor Arguments (ABI-encoded)
              </label>
              <textarea
                id="constructorArguments"
                value={formData.constructorArguments}
                onChange={(e) => updateFormData('constructorArguments', e.target.value)}
                rows={3}
                placeholder="0x000000000000000000000000..."
                className={classNames(
                  'w-full rounded-lg bg-stone-800 border px-4 py-3 text-white font-mono text-sm',
                  'focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-y',
                  errors.constructorArguments ? 'border-red-500' : 'border-stone-700'
                )}
              />
              <p className="mt-1 text-xs text-stone-500">
                Leave empty if the contract has no constructor or constructor has no parameters
              </p>
              {errors.constructorArguments && (
                <p className="mt-2 text-sm text-red-400 flex items-center gap-1">
                  <ExclamationCircleIcon className="h-4 w-4" />
                  {errors.constructorArguments}
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
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Contract Name</p>
                  <p className="text-sm text-white">{formData.contractName}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Compiler</p>
                  <p className="text-sm text-white">v{formData.compilerVersion}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Optimization</p>
                  <p className="text-sm text-white">
                    {formData.optimization ? `Enabled (${formData.optimizationRuns} runs)` : 'Disabled'}
                  </p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">EVM Version</p>
                  <p className="text-sm text-white capitalize">{formData.evmVersion}</p>
                </div>
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">License</p>
                  <p className="text-sm text-white uppercase">{formData.license}</p>
                </div>
              </div>

              <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Source Code</p>
                <p className="text-sm text-stone-400">
                  {formData.sourceCode.length.toLocaleString()} characters
                </p>
              </div>

              {formData.constructorArguments && (
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-1">Constructor Arguments</p>
                  <p className="text-sm text-white font-mono truncate">{formData.constructorArguments}</p>
                </div>
              )}

              {formData.libraries.length > 0 && (
                <div className="p-4 rounded-lg bg-stone-800/50 border border-stone-700">
                  <p className="text-xs text-stone-500 uppercase tracking-wide mb-2">Libraries</p>
                  {formData.libraries.map((lib, i) => (
                    <div key={i} className="text-sm">
                      <span className="text-white">{lib.name}</span>
                      <span className="text-stone-500 mx-2">→</span>
                      <span className="text-stone-400 font-mono">{lib.address}</span>
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
                      className="inline-flex items-center gap-1 text-sm text-pyrax-400 hover:text-pyrax-300 mt-2"
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
                className="w-full flex items-center justify-center gap-2 rounded-lg pyrax-gradient px-4 py-3 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50 transition-opacity"
              >
                {verifying ? (
                  <>
                    <ArrowPathIcon className="h-5 w-5 animate-spin" />
                    Verifying...
                  </>
                ) : (
                  <>
                    <ShieldCheckIcon className="h-5 w-5" />
                    Verify Contract
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
              className="inline-flex items-center gap-2 rounded-lg pyrax-gradient px-4 py-2 text-sm font-semibold text-white hover:opacity-90 transition-opacity"
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
