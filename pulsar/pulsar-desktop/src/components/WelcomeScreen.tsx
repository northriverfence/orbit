import { useState } from 'react';

interface WelcomeScreenProps {
  onComplete: () => void;
  onSkip: () => void;
}

type OnboardingStep = 'welcome' | 'features' | 'analytics' | 'quickstart' | 'complete';

export function WelcomeScreen({ onComplete, onSkip }: WelcomeScreenProps) {
  const [currentStep, setCurrentStep] = useState<OnboardingStep>('welcome');
  const [analyticsConsent, setAnalyticsConsent] = useState(false);

  const handleNext = () => {
    const steps: OnboardingStep[] = ['welcome', 'features', 'analytics', 'quickstart', 'complete'];
    const currentIndex = steps.indexOf(currentStep);
    if (currentIndex < steps.length - 1) {
      setCurrentStep(steps[currentIndex + 1]);
    }
  };

  const handleBack = () => {
    const steps: OnboardingStep[] = ['welcome', 'features', 'analytics', 'quickstart', 'complete'];
    const currentIndex = steps.indexOf(currentStep);
    if (currentIndex > 0) {
      setCurrentStep(steps[currentIndex - 1]);
    }
  };

  const handleFinish = () => {
    // Save analytics preference
    localStorage.setItem('analytics_enabled', String(analyticsConsent));
    localStorage.setItem('onboarding_completed', 'true');
    onComplete();
  };

  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-4xl mx-4 overflow-hidden">
        {/* Progress Bar */}
        <div className="h-2 bg-gray-200 dark:bg-gray-700">
          <div
            className="h-full bg-blue-600 transition-all duration-300"
            style={{
              width: `${
                currentStep === 'welcome'
                  ? 0
                  : currentStep === 'features'
                  ? 25
                  : currentStep === 'analytics'
                  ? 50
                  : currentStep === 'quickstart'
                  ? 75
                  : 100
              }%`,
            }}
          />
        </div>

        {/* Content */}
        <div className="p-12">
          {currentStep === 'welcome' && (
            <div className="text-center space-y-6">
              <div className="text-6xl mb-4">🚀</div>
              <h1 className="text-4xl font-bold text-gray-900 dark:text-white">
                Welcome to Pulsar Desktop!
              </h1>
              <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
                A modern SSH terminal with advanced features designed for developers, sysadmins, and power users.
              </p>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-8">
                <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <div className="text-3xl mb-2">📱</div>
                  <div className="font-semibold text-sm text-gray-900 dark:text-white">Multi-Session</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">Tabs & splits</div>
                </div>
                <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                  <div className="text-3xl mb-2">🔒</div>
                  <div className="font-semibold text-sm text-gray-900 dark:text-white">Secure Vault</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">Credential manager</div>
                </div>
                <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                  <div className="text-3xl mb-2">⚡</div>
                  <div className="font-semibold text-sm text-gray-900 dark:text-white">Fast Transfers</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">WebTransport</div>
                </div>
                <div className="p-4 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                  <div className="text-3xl mb-2">⌨️</div>
                  <div className="font-semibold text-sm text-gray-900 dark:text-white">Keyboard First</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">Shortcuts & palette</div>
                </div>
              </div>
            </div>
          )}

          {currentStep === 'features' && (
            <div className="space-y-8">
              <h2 className="text-3xl font-bold text-gray-900 dark:text-white text-center mb-8">
                Key Features
              </h2>

              <div className="space-y-6">
                <div className="flex gap-4">
                  <div className="flex-shrink-0 w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center text-2xl">
                    ⌨️
                  </div>
                  <div>
                    <h3 className="font-semibold text-lg text-gray-900 dark:text-white">
                      Command Palette (Ctrl/Cmd+K)
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                      Quick access to all features without leaving the keyboard. Search, navigate, and execute commands instantly.
                    </p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="flex-shrink-0 w-12 h-12 bg-purple-100 dark:bg-purple-900/30 rounded-full flex items-center justify-center text-2xl">
                    💾
                  </div>
                  <div>
                    <h3 className="font-semibold text-lg text-gray-900 dark:text-white">
                      Session Restoration
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                      Never lose your work. Sessions are automatically saved and can be restored when you relaunch the app.
                    </p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="flex-shrink-0 w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center text-2xl">
                    🔐
                  </div>
                  <div>
                    <h3 className="font-semibold text-lg text-gray-900 dark:text-white">
                      Secure Credential Vault
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                      Store SSH keys and credentials securely. Master password protection with encrypted storage.
                    </p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="flex-shrink-0 w-12 h-12 bg-orange-100 dark:bg-orange-900/30 rounded-full flex items-center justify-center text-2xl">
                    📁
                  </div>
                  <div>
                    <h3 className="font-semibold text-lg text-gray-900 dark:text-white">
                      File Transfer
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                      Upload and download files with progress tracking. Drag and drop support with resume capability.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {currentStep === 'analytics' && (
            <div className="space-y-6 max-w-2xl mx-auto">
              <div className="text-center mb-6">
                <div className="text-5xl mb-4">📊</div>
                <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  Help Us Improve
                </h2>
                <p className="text-gray-600 dark:text-gray-400">
                  We'd love to understand how you use Pulsar to make it better for everyone.
                </p>
              </div>

              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
                <h3 className="font-semibold text-lg text-gray-900 dark:text-white mb-4">
                  Privacy-First Analytics
                </h3>

                <div className="space-y-3 text-sm text-gray-700 dark:text-gray-300">
                  <div className="flex items-start gap-2">
                    <svg className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    <span><strong>Opt-in only</strong> - Analytics disabled by default</span>
                  </div>
                  <div className="flex items-start gap-2">
                    <svg className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    <span><strong>Anonymous</strong> - No personal information collected</span>
                  </div>
                  <div className="flex items-start gap-2">
                    <svg className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    <span><strong>Transparent</strong> - See exactly what's collected</span>
                  </div>
                  <div className="flex items-start gap-2">
                    <svg className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    <span><strong>Your control</strong> - Disable anytime in settings</span>
                  </div>
                </div>
              </div>

              <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
                <h4 className="font-semibold text-gray-900 dark:text-white mb-2">
                  What We DON'T Collect
                </h4>
                <div className="text-sm text-gray-700 dark:text-gray-300 space-y-1">
                  <p>❌ SSH credentials or passwords</p>
                  <p>❌ Host information or IP addresses</p>
                  <p>❌ Terminal commands or file contents</p>
                  <p>❌ Personal identifiable information</p>
                </div>
              </div>

              <label className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                <input
                  type="checkbox"
                  checked={analyticsConsent}
                  onChange={(e) => setAnalyticsConsent(e.target.checked)}
                  className="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="text-gray-900 dark:text-white font-medium">
                  Yes, I'd like to help improve Pulsar by sharing anonymous usage data
                </span>
              </label>
            </div>
          )}

          {currentStep === 'quickstart' && (
            <div className="space-y-6">
              <div className="text-center mb-8">
                <div className="text-5xl mb-4">🎓</div>
                <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
                  Quick Start Guide
                </h2>
                <p className="text-gray-600 dark:text-gray-400">
                  Get up and running in minutes
                </p>
              </div>

              <div className="grid md:grid-cols-2 gap-6">
                <div className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 p-6 rounded-xl border border-blue-200 dark:border-blue-800">
                  <div className="text-2xl mb-3">1️⃣</div>
                  <h3 className="font-semibold text-lg text-gray-900 dark:text-white mb-2">
                    Create Your First Connection
                  </h3>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                    Click "New Connection" or press Ctrl/Cmd+N to get started.
                  </p>
                  <code className="text-xs bg-white/50 dark:bg-black/20 px-2 py-1 rounded">
                    Ctrl/Cmd + N
                  </code>
                </div>

                <div className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 p-6 rounded-xl border border-purple-200 dark:border-purple-800">
                  <div className="text-2xl mb-3">2️⃣</div>
                  <h3 className="font-semibold text-lg text-gray-900 dark:text-white mb-2">
                    Open Command Palette
                  </h3>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                    Press Ctrl/Cmd+K to access all features instantly.
                  </p>
                  <code className="text-xs bg-white/50 dark:bg-black/20 px-2 py-1 rounded">
                    Ctrl/Cmd + K
                  </code>
                </div>

                <div className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 p-6 rounded-xl border border-green-200 dark:border-green-800">
                  <div className="text-2xl mb-3">3️⃣</div>
                  <h3 className="font-semibold text-lg text-gray-900 dark:text-white mb-2">
                    View Keyboard Shortcuts
                  </h3>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                    Press ? to see all available shortcuts.
                  </p>
                  <code className="text-xs bg-white/50 dark:bg-black/20 px-2 py-1 rounded">
                    Shift + ?
                  </code>
                </div>

                <div className="bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 p-6 rounded-xl border border-orange-200 dark:border-orange-800">
                  <div className="text-2xl mb-3">4️⃣</div>
                  <h3 className="font-semibold text-lg text-gray-900 dark:text-white mb-2">
                    Customize Settings
                  </h3>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                    Open settings to personalize your experience.
                  </p>
                  <code className="text-xs bg-white/50 dark:bg-black/20 px-2 py-1 rounded">
                    Ctrl/Cmd + ,
                  </code>
                </div>
              </div>

              <div className="bg-gray-50 dark:bg-gray-700/50 p-6 rounded-lg mt-6">
                <h4 className="font-semibold text-gray-900 dark:text-white mb-3">
                  📚 Additional Resources
                </h4>
                <ul className="space-y-2 text-sm text-gray-700 dark:text-gray-300">
                  <li>• <a href="#" className="text-blue-600 dark:text-blue-400 hover:underline">User Guide</a> - Comprehensive documentation</li>
                  <li>• <a href="#" className="text-blue-600 dark:text-blue-400 hover:underline">Video Tutorials</a> - Learn by watching</li>
                  <li>• <a href="#" className="text-blue-600 dark:text-blue-400 hover:underline">Discord Community</a> - Get help and share tips</li>
                </ul>
              </div>
            </div>
          )}

          {currentStep === 'complete' && (
            <div className="text-center space-y-6">
              <div className="text-6xl mb-4">🎉</div>
              <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
                You're All Set!
              </h2>
              <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
                Welcome to Pulsar Desktop. We hope you enjoy using it as much as we enjoyed building it.
              </p>

              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6 max-w-xl mx-auto">
                <p className="text-sm text-gray-700 dark:text-gray-300">
                  <strong>Beta Tester?</strong> Thank you for helping us improve! Please share your feedback at any time using <strong>Menu → Help → Send Feedback</strong> or press <strong>Ctrl/Cmd+Shift+F</strong>.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
          <button
            onClick={onSkip}
            className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
          >
            Skip Tour
          </button>

          <div className="flex gap-3">
            {currentStep !== 'welcome' && currentStep !== 'complete' && (
              <button
                onClick={handleBack}
                className="px-6 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                Back
              </button>
            )}

            {currentStep !== 'complete' ? (
              <button
                onClick={handleNext}
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                {currentStep === 'quickstart' ? 'Finish' : 'Next'}
              </button>
            ) : (
              <button
                onClick={handleFinish}
                className="px-8 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-semibold"
              >
                Let's Go! 🚀
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
