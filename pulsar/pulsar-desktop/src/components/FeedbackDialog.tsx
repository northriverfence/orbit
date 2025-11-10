import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface FeedbackDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

type FeedbackType = 'bug' | 'feature' | 'question' | 'other';

interface FeedbackData {
  type: FeedbackType;
  subject: string;
  description: string;
  email: string;
  includeScreenshot: boolean;
  includeLogs: boolean;
}

export function FeedbackDialog({ isOpen, onClose }: FeedbackDialogProps) {
  const [feedbackType, setFeedbackType] = useState<FeedbackType>('bug');
  const [subject, setSubject] = useState('');
  const [description, setDescription] = useState('');
  const [email, setEmail] = useState('');
  const [includeScreenshot, setIncludeScreenshot] = useState(false);
  const [includeLogs, setIncludeLogs] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitSuccess, setSubmitSuccess] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setSubmitError(null);

    try {
      const feedbackData: FeedbackData = {
        type: feedbackType,
        subject,
        description,
        email,
        includeScreenshot,
        includeLogs,
      };

      // Send feedback via Tauri command
      await invoke('send_feedback', { feedback: feedbackData });

      setSubmitSuccess(true);
      setTimeout(() => {
        onClose();
        resetForm();
      }, 2000);
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : 'Failed to send feedback');
    } finally {
      setIsSubmitting(false);
    }
  };

  const resetForm = () => {
    setFeedbackType('bug');
    setSubject('');
    setDescription('');
    setEmail('');
    setIncludeScreenshot(false);
    setIncludeLogs(true);
    setSubmitError(null);
    setSubmitSuccess(false);
  };

  const handleClose = () => {
    if (!isSubmitting) {
      onClose();
      resetForm();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Send Feedback
          </h2>
          <button
            onClick={handleClose}
            disabled={isSubmitting}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors disabled:opacity-50"
            aria-label="Close feedback dialog"
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-140px)]">
          {submitSuccess ? (
            <div className="text-center py-8">
              <div className="mb-4 text-green-600 dark:text-green-400">
                <svg className="w-16 h-16 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                Feedback Submitted!
              </h3>
              <p className="text-gray-600 dark:text-gray-400">
                Thank you for helping us improve Pulsar Desktop.
              </p>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Feedback Type */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Feedback Type
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {[
                    { value: 'bug', label: '🐛 Bug Report', desc: 'Something isn\'t working' },
                    { value: 'feature', label: '💡 Feature Request', desc: 'Suggest an idea' },
                    { value: 'question', label: '❓ Question', desc: 'Need help or clarification' },
                    { value: 'other', label: '💬 Other', desc: 'General feedback' },
                  ].map((type) => (
                    <button
                      key={type.value}
                      type="button"
                      onClick={() => setFeedbackType(type.value as FeedbackType)}
                      className={`p-3 rounded-lg border-2 text-left transition-colors ${
                        feedbackType === type.value
                          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                          : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                      }`}
                    >
                      <div className="font-medium text-sm text-gray-900 dark:text-white">
                        {type.label}
                      </div>
                      <div className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                        {type.desc}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Subject */}
              <div>
                <label htmlFor="subject" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Subject
                </label>
                <input
                  id="subject"
                  type="text"
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  required
                  placeholder="Brief summary of your feedback"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                />
              </div>

              {/* Description */}
              <div>
                <label htmlFor="description" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Description
                </label>
                <textarea
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  required
                  rows={6}
                  placeholder={
                    feedbackType === 'bug'
                      ? 'Please describe what happened, expected behavior, and steps to reproduce...'
                      : feedbackType === 'feature'
                      ? 'Describe the feature you\'d like to see and why it would be useful...'
                      : 'Tell us more about your feedback...'
                  }
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white resize-none"
                />
              </div>

              {/* Email (optional) */}
              <div>
                <label htmlFor="email" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Email (optional)
                </label>
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="your.email@example.com"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Provide your email if you'd like us to follow up with you
                </p>
              </div>

              {/* Attachments */}
              <div className="space-y-3">
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={includeScreenshot}
                    onChange={(e) => setIncludeScreenshot(e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    Include screenshot
                  </span>
                </label>

                {feedbackType === 'bug' && (
                  <label className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      checked={includeLogs}
                      onChange={(e) => setIncludeLogs(e.target.checked)}
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700 dark:text-gray-300">
                      Include diagnostic logs (recommended for bug reports)
                    </span>
                  </label>
                )}

                <p className="text-xs text-gray-500 dark:text-gray-400">
                  No personal data (credentials, file contents, etc.) will be included
                </p>
              </div>

              {/* Error Message */}
              {submitError && (
                <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                  <p className="text-sm text-red-600 dark:text-red-400">
                    {submitError}
                  </p>
                </div>
              )}
            </form>
          )}
        </div>

        {/* Footer */}
        {!submitSuccess && (
          <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              onClick={handleClose}
              disabled={isSubmitting}
              className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={isSubmitting || !subject || !description}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
            >
              {isSubmitting ? (
                <>
                  <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                      fill="none"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Sending...
                </>
              ) : (
                'Send Feedback'
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
