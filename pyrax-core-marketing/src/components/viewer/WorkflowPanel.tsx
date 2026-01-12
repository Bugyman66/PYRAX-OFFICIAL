'use client';

import { useState, useEffect } from 'react';
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  Clock,
  ChevronDown,
  ChevronUp,
  MessageSquare,
  Loader2,
  Play,
  RotateCcw,
  User,
  History,
} from 'lucide-react';

interface WorkflowStep {
  name: string;
  allowedRoles: string[];
  allowedUsers: string[];
  approvalRule: 'any' | 'all' | 'single';
}

interface Decision {
  id: string;
  stepIndex: number;
  decisionType: 'Approve' | 'NeedsChanges' | 'Reject';
  note: string | null;
  decidedBy: { id: string; name: string | null; email: string };
  createdAt: string;
}

interface AuditEvent {
  id: string;
  action: string;
  actor: { id: string; name: string | null; email: string } | null;
  payload: Record<string, unknown> | null;
  createdAt: string;
}

interface WorkflowStatus {
  instance: {
    id: string;
    status: 'Active' | 'Completed' | 'Cancelled';
    currentStepIndex: number;
    createdAt: string;
  };
  template: {
    id: string;
    name: string;
    steps: WorkflowStep[];
  };
  decisions: Decision[];
  currentStep: WorkflowStep | null;
  progress: {
    completedSteps: number;
    totalSteps: number;
    percentComplete: number;
  };
  proof: {
    id: string;
    title: string;
    status: string;
  };
  canDecide: boolean;
  canDecideReason?: string;
  auditTrail: AuditEvent[];
  awaitingReReview?: boolean;
}

interface WorkflowTemplate {
  id: string;
  name: string;
  description: string | null;
  steps: WorkflowStep[];
  isDefault: boolean;
}

interface WorkflowPanelProps {
  proofId: string;
  onWorkflowChange?: () => void;
}

export default function WorkflowPanel({ proofId, onWorkflowChange }: WorkflowPanelProps) {
  const [workflowStatus, setWorkflowStatus] = useState<WorkflowStatus | null>(null);
  const [templates, setTemplates] = useState<WorkflowTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(false);
  const [showDecisionModal, setShowDecisionModal] = useState(false);
  const [showStartModal, setShowStartModal] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    fetchWorkflowData();
  }, [proofId]);

  async function fetchWorkflowData() {
    try {
      setLoading(true);
      setError(null);

      const instancesRes = await fetch(`/api/workflow-instances?proofId=${proofId}`);
      if (!instancesRes.ok) throw new Error('Failed to fetch workflow');
      const instances = await instancesRes.json();

      if (instances.length > 0) {
        const statusRes = await fetch(`/api/workflow-instances/${instances[0].id}`);
        if (!statusRes.ok) throw new Error('Failed to fetch workflow status');
        const status = await statusRes.json();
        setWorkflowStatus(status);
      } else {
        setWorkflowStatus(null);
        const templatesRes = await fetch('/api/workflow-templates');
        if (templatesRes.ok) {
          const templatesData = await templatesRes.json();
          setTemplates(templatesData);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load workflow');
    } finally {
      setLoading(false);
    }
  }

  async function startWorkflow(templateId: string) {
    try {
      setSubmitting(true);
      const res = await fetch('/api/workflow-instances', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ proofId, templateId }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to start workflow');
      }

      setShowStartModal(false);
      await fetchWorkflowData();
      onWorkflowChange?.();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to start workflow');
    } finally {
      setSubmitting(false);
    }
  }

  async function makeDecision(decisionType: 'Approve' | 'NeedsChanges' | 'Reject', note: string) {
    if (!workflowStatus) return;

    try {
      setSubmitting(true);
      const res = await fetch(`/api/workflow-instances/${workflowStatus.instance.id}/decisions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ decisionType, note: note || null }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to submit decision');
      }

      setShowDecisionModal(false);
      await fetchWorkflowData();
      onWorkflowChange?.();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to submit decision');
    } finally {
      setSubmitting(false);
    }
  }

  async function cancelWorkflow() {
    if (!workflowStatus) return;
    if (!confirm('Are you sure you want to cancel this workflow? The proof will return to Draft status.')) return;

    try {
      setSubmitting(true);
      const res = await fetch(`/api/workflow-instances/${workflowStatus.instance.id}`, {
        method: 'DELETE',
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to cancel workflow');
      }

      await fetchWorkflowData();
      onWorkflowChange?.();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to cancel workflow');
    } finally {
      setSubmitting(false);
    }
  }

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <Loader2 className="w-6 h-6 text-pyrax-500 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-red-400 text-sm">
          {error}
        </div>
      </div>
    );
  }

  if (!workflowStatus) {
    return (
      <div className="p-4 space-y-4">
        <div className="text-center py-6">
          <div className="w-12 h-12 bg-stone-800 rounded-full flex items-center justify-center mx-auto mb-3">
            <Play className="w-6 h-6 text-stone-500" />
          </div>
          <h3 className="font-medium text-stone-200 mb-1">No Active Workflow</h3>
          <p className="text-sm text-stone-400 mb-4">Start a review workflow to collect approvals</p>
          <button
            onClick={() => setShowStartModal(true)}
            className="px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-lg transition-colors text-sm"
          >
            Start Workflow
          </button>
        </div>

        {showStartModal && (
          <StartWorkflowModal
            templates={templates}
            onStart={startWorkflow}
            onClose={() => setShowStartModal(false)}
            submitting={submitting}
          />
        )}
      </div>
    );
  }

  const { instance, template, decisions, currentStep, progress } = workflowStatus;

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b border-stone-800">
        <div className="flex items-center justify-between mb-3">
          <h3 className="font-medium text-stone-100">{template.name}</h3>
          <StatusBadge status={instance.status} />
        </div>

        <div className="mb-3">
          <div className="flex items-center justify-between text-xs text-stone-400 mb-1">
            <span>Progress</span>
            <span>{progress.completedSteps} / {progress.totalSteps} steps</span>
          </div>
          <div className="h-2 bg-stone-800 rounded-full overflow-hidden">
            <div
              className={`h-full transition-all duration-300 ${
                instance.status === 'Completed' ? 'bg-green-500' :
                instance.status === 'Cancelled' ? 'bg-red-500' :
                'bg-pyrax-500'
              }`}
              style={{ width: `${progress.percentComplete}%` }}
            />
          </div>
        </div>

        {instance.status === 'Active' && currentStep && (
          <div className="bg-stone-800/50 rounded-lg p-3">
            <div className="text-xs text-stone-400 mb-1">Current Step</div>
            <div className="font-medium text-stone-100">{currentStep.name}</div>
            <div className="text-xs text-stone-400 mt-1">
              {currentStep.allowedRoles.join(', ')} • {
                currentStep.approvalRule === 'any' ? 'Any approval advances' :
                currentStep.approvalRule === 'all' ? 'All must approve' :
                'Single approver'
              }
            </div>
            {decisions.some(d => d.decisionType === 'NeedsChanges') && (
              <div className="mt-2 flex items-center gap-2 text-xs text-yellow-400 bg-yellow-500/10 px-2 py-1 rounded">
                <RotateCcw className="w-3 h-3" />
                <span>Re-review cycle - new version available</span>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        <div className="text-xs font-medium text-stone-400 uppercase tracking-wider mb-2">
          Workflow Steps
        </div>
        {template.steps.map((step, index) => {
          const stepDecisions = decisions.filter(d => d.stepIndex === index);
          const isCurrentStep = index === instance.currentStepIndex && instance.status === 'Active';
          const isCompleted = index < instance.currentStepIndex || instance.status === 'Completed';
          const hasRejection = stepDecisions.some(d => d.decisionType === 'Reject');
          const hasNeedsChanges = stepDecisions.some(d => d.decisionType === 'NeedsChanges');

          return (
            <div
              key={index}
              className={`rounded-lg border p-3 transition-colors ${
                isCurrentStep
                  ? 'bg-pyrax-500/10 border-pyrax-500/30'
                  : isCompleted
                  ? 'bg-stone-800/30 border-stone-700'
                  : 'bg-stone-900 border-stone-800'
              }`}
            >
              <div className="flex items-start gap-3">
                <div className={`w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0 ${
                  hasRejection ? 'bg-red-500/20 text-red-400' :
                  hasNeedsChanges ? 'bg-yellow-500/20 text-yellow-400' :
                  isCompleted ? 'bg-green-500/20 text-green-400' :
                  isCurrentStep ? 'bg-pyrax-500/20 text-pyrax-400' :
                  'bg-stone-700 text-stone-400'
                }`}>
                  {hasRejection ? <XCircle className="w-4 h-4" /> :
                   hasNeedsChanges ? <AlertTriangle className="w-4 h-4" /> :
                   isCompleted ? <CheckCircle className="w-4 h-4" /> :
                   isCurrentStep ? <Clock className="w-4 h-4" /> :
                   <span className="text-xs">{index + 1}</span>}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-stone-200 text-sm">{step.name}</div>
                  {stepDecisions.length > 0 && (
                    <div className="mt-2 space-y-1">
                      {stepDecisions.map(decision => (
                        <div key={decision.id} className="flex items-center gap-2 text-xs">
                          <DecisionIcon type={decision.decisionType} />
                          <span className="text-stone-400">
                            {decision.decidedBy.name || decision.decidedBy.email}
                          </span>
                          {decision.note && (
                            <span className="text-stone-500 truncate" title={decision.note}>
                              - {decision.note}
                            </span>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {instance.status === 'Active' && (
        <div className="p-4 border-t border-stone-800 space-y-3">
          {workflowStatus.canDecide ? (
            <button
              onClick={() => setShowDecisionModal(true)}
              className="w-full py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-lg transition-colors font-medium"
            >
              {decisions.some(d => d.decisionType === 'NeedsChanges') ? 'Re-Review & Decide' : 'Make Decision'}
            </button>
          ) : (
            <div className="text-center text-sm text-stone-400 py-2">
              {workflowStatus.canDecideReason || 'You cannot make a decision at this time'}
            </div>
          )}
          
          <button
            onClick={() => setShowHistory(!showHistory)}
            className="w-full flex items-center justify-center gap-2 py-2 text-stone-400 hover:text-stone-300 text-sm"
          >
            <History className="w-4 h-4" />
            {showHistory ? 'Hide' : 'Show'} Activity
            {showHistory ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>

          {showHistory && (
            <div className="bg-stone-800/50 rounded-lg p-3 max-h-48 overflow-y-auto space-y-2">
              {workflowStatus.auditTrail.map(event => (
                <div key={event.id} className="text-xs">
                  <div className="flex items-center gap-2 text-stone-300">
                    <User className="w-3 h-3" />
                    <span>{event.actor?.name || event.actor?.email || 'System'}</span>
                    <span className="text-stone-500">
                      {formatAction(event.action)}
                    </span>
                  </div>
                  <div className="text-stone-500 ml-5">
                    {new Date(event.createdAt).toLocaleString()}
                  </div>
                </div>
              ))}
            </div>
          )}

          <button
            onClick={cancelWorkflow}
            disabled={submitting}
            className="w-full flex items-center justify-center gap-2 py-2 text-red-400 hover:text-red-300 text-sm"
          >
            <XCircle className="w-4 h-4" />
            Cancel Workflow
          </button>
        </div>
      )}

      {showDecisionModal && (
        <DecisionModal
          currentStep={currentStep}
          onDecision={makeDecision}
          onClose={() => setShowDecisionModal(false)}
          submitting={submitting}
        />
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: 'Active' | 'Completed' | 'Cancelled' }) {
  const config = {
    Active: { bg: 'bg-blue-500/20', text: 'text-blue-400', label: 'In Progress' },
    Completed: { bg: 'bg-green-500/20', text: 'text-green-400', label: 'Approved' },
    Cancelled: { bg: 'bg-red-500/20', text: 'text-red-400', label: 'Cancelled' },
  };
  const { bg, text, label } = config[status];
  return (
    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${bg} ${text}`}>
      {label}
    </span>
  );
}

function DecisionIcon({ type }: { type: 'Approve' | 'NeedsChanges' | 'Reject' }) {
  switch (type) {
    case 'Approve':
      return <CheckCircle className="w-3 h-3 text-green-400" />;
    case 'NeedsChanges':
      return <RotateCcw className="w-3 h-3 text-yellow-400" />;
    case 'Reject':
      return <XCircle className="w-3 h-3 text-red-400" />;
  }
}

function formatAction(action: string): string {
  const actions: Record<string, string> = {
    created: 'started the workflow',
    step_advanced: 'advanced to next step',
    completed: 'completed the workflow',
    rejected: 'rejected the proof',
    cancelled: 'cancelled the workflow',
    returned_for_changes: 'requested changes',
  };
  return actions[action] || action;
}

interface StartWorkflowModalProps {
  templates: WorkflowTemplate[];
  onStart: (templateId: string) => void;
  onClose: () => void;
  submitting: boolean;
}

function StartWorkflowModal({ templates, onStart, onClose, submitting }: StartWorkflowModalProps) {
  const [selectedTemplate, setSelectedTemplate] = useState<string>(
    templates.find(t => t.isDefault)?.id || templates[0]?.id || ''
  );

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-stone-900 border border-stone-800 rounded-xl w-full max-w-md">
        <div className="p-4 border-b border-stone-800">
          <h3 className="font-medium text-stone-100">Start Review Workflow</h3>
        </div>
        <div className="p-4 space-y-4">
          {templates.length === 0 ? (
            <p className="text-stone-400 text-sm">No workflow templates available. Please create one first.</p>
          ) : (
            <>
              <div>
                <label className="block text-sm text-stone-300 mb-2">Select Template</label>
                <div className="space-y-2">
                  {templates.map(template => (
                    <label
                      key={template.id}
                      className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                        selectedTemplate === template.id
                          ? 'bg-pyrax-500/10 border-pyrax-500/30'
                          : 'bg-stone-800 border-stone-700 hover:border-stone-600'
                      }`}
                    >
                      <input
                        type="radio"
                        name="template"
                        value={template.id}
                        checked={selectedTemplate === template.id}
                        onChange={e => setSelectedTemplate(e.target.value)}
                        className="mt-1"
                      />
                      <div>
                        <div className="font-medium text-stone-200 flex items-center gap-2">
                          {template.name}
                          {template.isDefault && (
                            <span className="text-xs text-pyrax-400">(Default)</span>
                          )}
                        </div>
                        {template.description && (
                          <p className="text-xs text-stone-400 mt-0.5">{template.description}</p>
                        )}
                        <p className="text-xs text-stone-500 mt-1">
                          {template.steps.length} step{template.steps.length !== 1 ? 's' : ''}
                        </p>
                      </div>
                    </label>
                  ))}
                </div>
              </div>
            </>
          )}
        </div>
        <div className="p-4 border-t border-stone-800 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-stone-400 hover:text-stone-300"
          >
            Cancel
          </button>
          <button
            onClick={() => selectedTemplate && onStart(selectedTemplate)}
            disabled={!selectedTemplate || submitting}
            className="flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white rounded-lg"
          >
            {submitting && <Loader2 className="w-4 h-4 animate-spin" />}
            Start Workflow
          </button>
        </div>
      </div>
    </div>
  );
}

interface DecisionModalProps {
  currentStep: WorkflowStep | null;
  onDecision: (type: 'Approve' | 'NeedsChanges' | 'Reject', note: string) => void;
  onClose: () => void;
  submitting: boolean;
}

function DecisionModal({ currentStep, onDecision, onClose, submitting }: DecisionModalProps) {
  const [selectedDecision, setSelectedDecision] = useState<'Approve' | 'NeedsChanges' | 'Reject' | null>(null);
  const [note, setNote] = useState('');

  const decisions = [
    { value: 'Approve' as const, label: 'Approve', icon: CheckCircle, color: 'text-green-400', bg: 'bg-green-500/10 border-green-500/30' },
    { value: 'NeedsChanges' as const, label: 'Needs Changes', icon: RotateCcw, color: 'text-yellow-400', bg: 'bg-yellow-500/10 border-yellow-500/30' },
    { value: 'Reject' as const, label: 'Reject', icon: XCircle, color: 'text-red-400', bg: 'bg-red-500/10 border-red-500/30' },
  ];

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-stone-900 border border-stone-800 rounded-xl w-full max-w-md">
        <div className="p-4 border-b border-stone-800">
          <h3 className="font-medium text-stone-100">Make Decision</h3>
          {currentStep && (
            <p className="text-sm text-stone-400 mt-1">Step: {currentStep.name}</p>
          )}
        </div>
        <div className="p-4 space-y-4">
          <div className="grid grid-cols-3 gap-2">
            {decisions.map(({ value, label, icon: Icon, color, bg }) => (
              <button
                key={value}
                onClick={() => setSelectedDecision(value)}
                className={`p-3 rounded-lg border text-center transition-colors ${
                  selectedDecision === value ? bg : 'bg-stone-800 border-stone-700 hover:border-stone-600'
                }`}
              >
                <Icon className={`w-6 h-6 mx-auto mb-1 ${selectedDecision === value ? color : 'text-stone-400'}`} />
                <span className={`text-xs font-medium ${selectedDecision === value ? color : 'text-stone-300'}`}>
                  {label}
                </span>
              </button>
            ))}
          </div>

          <div>
            <label className="block text-sm text-stone-300 mb-1">
              Note {selectedDecision !== 'Approve' && <span className="text-stone-500">(recommended)</span>}
            </label>
            <textarea
              value={note}
              onChange={e => setNote(e.target.value)}
              placeholder={
                selectedDecision === 'NeedsChanges'
                  ? 'Describe what changes are needed...'
                  : selectedDecision === 'Reject'
                  ? 'Explain why this is being rejected...'
                  : 'Add an optional note...'
              }
              rows={3}
              className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-none text-sm"
            />
          </div>
        </div>
        <div className="p-4 border-t border-stone-800 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-stone-400 hover:text-stone-300"
          >
            Cancel
          </button>
          <button
            onClick={() => selectedDecision && onDecision(selectedDecision, note)}
            disabled={!selectedDecision || submitting}
            className="flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white rounded-lg"
          >
            {submitting && <Loader2 className="w-4 h-4 animate-spin" />}
            Submit Decision
          </button>
        </div>
      </div>
    </div>
  );
}
