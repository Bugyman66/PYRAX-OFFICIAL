'use client';

import { useState, useEffect } from 'react';
import { 
  Plus, 
  Edit2, 
  Trash2, 
  Copy, 
  Star, 
  ChevronDown, 
  ChevronUp,
  GripVertical,
  Users,
  CheckCircle,
  AlertCircle,
  Loader2,
  X
} from 'lucide-react';

interface WorkflowStep {
  name: string;
  allowedRoles: string[];
  allowedUsers: string[];
  approvalRule: 'any' | 'all' | 'single';
}

interface WorkflowFinalRule {
  type: 'single' | 'majority' | 'unanimous';
  description?: string;
}

interface WorkflowTemplate {
  id: string;
  name: string;
  description: string | null;
  department: { id: string; name: string };
  steps: WorkflowStep[];
  finalRule: WorkflowFinalRule;
  isDefault: boolean;
  instanceCount: number;
  createdAt: string;
  updatedAt: string;
}

const ROLE_OPTIONS = [
  { value: 'OrgAdmin', label: 'Organization Admin' },
  { value: 'DepartmentHead', label: 'Department Head' },
  { value: 'Employee', label: 'Employee' },
];

const APPROVAL_RULES = [
  { value: 'any', label: 'Any (First approval advances)', description: 'The first person to approve moves the workflow forward' },
  { value: 'all', label: 'All (Everyone must approve)', description: 'All allowed users must approve before advancing' },
  { value: 'single', label: 'Single (Designated approver)', description: 'Only one designated person can approve' },
];

export default function WorkflowsPage() {
  const [templates, setTemplates] = useState<WorkflowTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showEditor, setShowEditor] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<WorkflowTemplate | null>(null);
  const [expandedTemplates, setExpandedTemplates] = useState<Set<string>>(new Set());

  useEffect(() => {
    fetchTemplates();
  }, []);

  async function fetchTemplates() {
    try {
      setLoading(true);
      const res = await fetch('/api/workflow-templates');
      if (!res.ok) throw new Error('Failed to fetch templates');
      const data = await res.json();
      setTemplates(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load templates');
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(templateId: string) {
    if (!confirm('Are you sure you want to delete this template?')) return;
    
    try {
      const res = await fetch(`/api/workflow-templates/${templateId}`, {
        method: 'DELETE',
      });
      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to delete template');
      }
      setTemplates(templates.filter(t => t.id !== templateId));
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete template');
    }
  }

  async function handleDuplicate(template: WorkflowTemplate) {
    try {
      const res = await fetch('/api/workflow-templates', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: `${template.name} (Copy)`,
          description: template.description,
          departmentId: template.department.id,
          steps: template.steps,
          finalRule: template.finalRule,
          isDefault: false,
        }),
      });
      if (!res.ok) throw new Error('Failed to duplicate template');
      const newTemplate = await res.json();
      setTemplates([...templates, newTemplate]);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to duplicate template');
    }
  }

  function toggleExpanded(templateId: string) {
    const newExpanded = new Set(expandedTemplates);
    if (newExpanded.has(templateId)) {
      newExpanded.delete(templateId);
    } else {
      newExpanded.add(templateId);
    }
    setExpandedTemplates(newExpanded);
  }

  function openEditor(template?: WorkflowTemplate) {
    setEditingTemplate(template || null);
    setShowEditor(true);
  }

  function closeEditor() {
    setShowEditor(false);
    setEditingTemplate(null);
  }

  function handleSaved(template: WorkflowTemplate) {
    if (editingTemplate) {
      setTemplates(templates.map(t => t.id === template.id ? template : t));
    } else {
      setTemplates([...templates, template]);
    }
    closeEditor();
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 text-pyrax-500 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-stone-100">Workflow Templates</h1>
          <p className="text-stone-400 mt-1">Create and manage approval workflows for your department</p>
        </div>
        <button
          onClick={() => openEditor()}
          className="flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-lg transition-colors"
        >
          <Plus className="w-5 h-5" />
          New Template
        </button>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <span className="text-red-400">{error}</span>
        </div>
      )}

      {templates.length === 0 ? (
        <div className="bg-stone-900 border border-stone-800 rounded-xl p-12 text-center">
          <div className="w-16 h-16 bg-stone-800 rounded-full flex items-center justify-center mx-auto mb-4">
            <Users className="w-8 h-8 text-stone-500" />
          </div>
          <h3 className="text-lg font-medium text-stone-200 mb-2">No workflow templates yet</h3>
          <p className="text-stone-400 mb-6">Create your first workflow template to start managing approvals</p>
          <button
            onClick={() => openEditor()}
            className="inline-flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-lg transition-colors"
          >
            <Plus className="w-5 h-5" />
            Create Template
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          {templates.map(template => (
            <div
              key={template.id}
              className="bg-stone-900 border border-stone-800 rounded-xl overflow-hidden"
            >
              <div className="p-4 flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <button
                    onClick={() => toggleExpanded(template.id)}
                    className="p-1 hover:bg-stone-800 rounded"
                  >
                    {expandedTemplates.has(template.id) ? (
                      <ChevronUp className="w-5 h-5 text-stone-400" />
                    ) : (
                      <ChevronDown className="w-5 h-5 text-stone-400" />
                    )}
                  </button>
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium text-stone-100">{template.name}</h3>
                      {template.isDefault && (
                        <span className="flex items-center gap-1 px-2 py-0.5 bg-pyrax-500/20 text-pyrax-400 text-xs rounded-full">
                          <Star className="w-3 h-3" />
                          Default
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-stone-400 mt-0.5">
                      {template.steps.length} step{template.steps.length !== 1 ? 's' : ''} • {template.instanceCount} active instance{template.instanceCount !== 1 ? 's' : ''}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => openEditor(template)}
                    className="p-2 hover:bg-stone-800 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <Edit2 className="w-4 h-4 text-stone-400" />
                  </button>
                  <button
                    onClick={() => handleDuplicate(template)}
                    className="p-2 hover:bg-stone-800 rounded-lg transition-colors"
                    title="Duplicate"
                  >
                    <Copy className="w-4 h-4 text-stone-400" />
                  </button>
                  <button
                    onClick={() => handleDelete(template.id)}
                    className="p-2 hover:bg-stone-800 rounded-lg transition-colors"
                    title="Delete"
                    disabled={template.instanceCount > 0}
                  >
                    <Trash2 className={`w-4 h-4 ${template.instanceCount > 0 ? 'text-stone-600' : 'text-stone-400'}`} />
                  </button>
                </div>
              </div>

              {expandedTemplates.has(template.id) && (
                <div className="border-t border-stone-800 p-4 bg-stone-950/50">
                  {template.description && (
                    <p className="text-stone-400 text-sm mb-4">{template.description}</p>
                  )}
                  <div className="space-y-3">
                    {template.steps.map((step, index) => (
                      <div
                        key={index}
                        className="flex items-center gap-4 p-3 bg-stone-900 rounded-lg"
                      >
                        <div className="w-8 h-8 bg-pyrax-500/20 text-pyrax-400 rounded-full flex items-center justify-center font-medium text-sm">
                          {index + 1}
                        </div>
                        <div className="flex-1">
                          <div className="font-medium text-stone-200">{step.name}</div>
                          <div className="text-sm text-stone-400 mt-0.5">
                            {step.allowedRoles.join(', ')} • {APPROVAL_RULES.find(r => r.value === step.approvalRule)?.label}
                          </div>
                        </div>
                        <CheckCircle className="w-5 h-5 text-stone-600" />
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {showEditor && (
        <TemplateEditor
          template={editingTemplate}
          onClose={closeEditor}
          onSaved={handleSaved}
        />
      )}
    </div>
  );
}

interface TemplateEditorProps {
  template: WorkflowTemplate | null;
  onClose: () => void;
  onSaved: (template: WorkflowTemplate) => void;
}

function TemplateEditor({ template, onClose, onSaved }: TemplateEditorProps) {
  const [name, setName] = useState(template?.name || '');
  const [description, setDescription] = useState(template?.description || '');
  const [steps, setSteps] = useState<WorkflowStep[]>(
    template?.steps || [{ name: '', allowedRoles: ['Employee'], allowedUsers: [], approvalRule: 'any' }]
  );
  const [isDefault, setIsDefault] = useState(template?.isDefault || false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function addStep() {
    setSteps([...steps, { name: '', allowedRoles: ['Employee'], allowedUsers: [], approvalRule: 'any' }]);
  }

  function removeStep(index: number) {
    if (steps.length <= 1) return;
    setSteps(steps.filter((_, i) => i !== index));
  }

  function updateStep(index: number, updates: Partial<WorkflowStep>) {
    setSteps(steps.map((step, i) => i === index ? { ...step, ...updates } : step));
  }

  function moveStep(index: number, direction: 'up' | 'down') {
    if (direction === 'up' && index === 0) return;
    if (direction === 'down' && index === steps.length - 1) return;
    
    const newSteps = [...steps];
    const targetIndex = direction === 'up' ? index - 1 : index + 1;
    [newSteps[index], newSteps[targetIndex]] = [newSteps[targetIndex], newSteps[index]];
    setSteps(newSteps);
  }

  function toggleRole(stepIndex: number, role: string) {
    const step = steps[stepIndex];
    const newRoles = step.allowedRoles.includes(role)
      ? step.allowedRoles.filter(r => r !== role)
      : [...step.allowedRoles, role];
    updateStep(stepIndex, { allowedRoles: newRoles });
  }

  async function handleSave() {
    if (!name.trim()) {
      setError('Name is required');
      return;
    }

    for (let i = 0; i < steps.length; i++) {
      if (!steps[i].name.trim()) {
        setError(`Step ${i + 1} requires a name`);
        return;
      }
      if (steps[i].allowedRoles.length === 0) {
        setError(`Step ${i + 1} requires at least one allowed role`);
        return;
      }
    }

    try {
      setSaving(true);
      setError(null);

      const url = template
        ? `/api/workflow-templates/${template.id}`
        : '/api/workflow-templates';
      
      const method = template ? 'PATCH' : 'POST';

      const body: Record<string, unknown> = {
          name: name.trim(),
          description: description.trim() || null,
          steps,
          finalRule: { type: 'single', description: 'Standard approval' },
          isDefault,
        };

      if (!template) {
        body.departmentId = undefined;
      }

      const res = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to save template');
      }

      const saved = await res.json();
      onSaved(saved);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save template');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-stone-900 border border-stone-800 rounded-xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-stone-100">
            {template ? 'Edit Template' : 'New Workflow Template'}
          </h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-stone-800 rounded-lg"
          >
            <X className="w-5 h-5 text-stone-400" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-6">
          {error && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-500" />
              <span className="text-red-400 text-sm">{error}</span>
            </div>
          )}

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-stone-300 mb-1">Template Name</label>
              <input
                type="text"
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder="e.g., Standard Review Process"
                className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-stone-300 mb-1">Description (Optional)</label>
              <textarea
                value={description}
                onChange={e => setDescription(e.target.value)}
                placeholder="Describe when this workflow should be used..."
                rows={2}
                className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-none"
              />
            </div>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={isDefault}
                onChange={e => setIsDefault(e.target.checked)}
                className="w-4 h-4 rounded border-stone-600 bg-stone-800 text-pyrax-500 focus:ring-pyrax-500"
              />
              <span className="text-sm text-stone-300">Set as default template for this department</span>
            </label>
          </div>

          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-stone-300">Workflow Steps</label>
              <button
                onClick={addStep}
                className="text-sm text-pyrax-400 hover:text-pyrax-300 flex items-center gap-1"
              >
                <Plus className="w-4 h-4" />
                Add Step
              </button>
            </div>

            <div className="space-y-3">
              {steps.map((step, index) => (
                <div
                  key={index}
                  className="bg-stone-800 border border-stone-700 rounded-lg p-4"
                >
                  <div className="flex items-start gap-3">
                    <div className="flex flex-col items-center gap-1 pt-1">
                      <button
                        onClick={() => moveStep(index, 'up')}
                        disabled={index === 0}
                        className="p-1 hover:bg-stone-700 rounded disabled:opacity-30"
                      >
                        <ChevronUp className="w-4 h-4 text-stone-400" />
                      </button>
                      <GripVertical className="w-4 h-4 text-stone-500" />
                      <button
                        onClick={() => moveStep(index, 'down')}
                        disabled={index === steps.length - 1}
                        className="p-1 hover:bg-stone-700 rounded disabled:opacity-30"
                      >
                        <ChevronDown className="w-4 h-4 text-stone-400" />
                      </button>
                    </div>

                    <div className="flex-1 space-y-3">
                      <div className="flex items-center gap-3">
                        <span className="w-6 h-6 bg-pyrax-500/20 text-pyrax-400 rounded-full flex items-center justify-center text-sm font-medium">
                          {index + 1}
                        </span>
                        <input
                          type="text"
                          value={step.name}
                          onChange={e => updateStep(index, { name: e.target.value })}
                          placeholder="Step name"
                          className="flex-1 px-3 py-1.5 bg-stone-900 border border-stone-600 rounded text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                        />
                        {steps.length > 1 && (
                          <button
                            onClick={() => removeStep(index)}
                            className="p-1.5 hover:bg-stone-700 rounded"
                          >
                            <X className="w-4 h-4 text-stone-400" />
                          </button>
                        )}
                      </div>

                      <div>
                        <label className="text-xs text-stone-400 mb-1 block">Allowed Roles</label>
                        <div className="flex flex-wrap gap-2">
                          {ROLE_OPTIONS.map(role => (
                            <button
                              key={role.value}
                              onClick={() => toggleRole(index, role.value)}
                              className={`px-2 py-1 text-xs rounded-full transition-colors ${
                                step.allowedRoles.includes(role.value)
                                  ? 'bg-pyrax-500/20 text-pyrax-400 border border-pyrax-500/30'
                                  : 'bg-stone-700 text-stone-400 border border-stone-600 hover:border-stone-500'
                              }`}
                            >
                              {role.label}
                            </button>
                          ))}
                        </div>
                      </div>

                      <div>
                        <label className="text-xs text-stone-400 mb-1 block">Approval Rule</label>
                        <select
                          value={step.approvalRule}
                          onChange={e => updateStep(index, { approvalRule: e.target.value as 'any' | 'all' | 'single' })}
                          className="w-full px-3 py-1.5 bg-stone-900 border border-stone-600 rounded text-stone-100 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                        >
                          {APPROVAL_RULES.map(rule => (
                            <option key={rule.value} value={rule.value}>
                              {rule.label}
                            </option>
                          ))}
                        </select>
                        <p className="text-xs text-stone-500 mt-1">
                          {APPROVAL_RULES.find(r => r.value === step.approvalRule)?.description}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="p-4 border-t border-stone-800 flex items-center justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-stone-400 hover:text-stone-300 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white rounded-lg transition-colors"
          >
            {saving && <Loader2 className="w-4 h-4 animate-spin" />}
            {template ? 'Save Changes' : 'Create Template'}
          </button>
        </div>
      </div>
    </div>
  );
}
