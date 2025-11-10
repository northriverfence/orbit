interface EmptyStateProps {
  icon?: string
  title: string
  description: string
  action?: {
    label: string
    onClick: () => void
  }
}

export default function EmptyState({
  icon = '📭',
  title,
  description,
  action,
}: EmptyStateProps) {
  return (
    <div className="flex items-center justify-center h-full py-12 animate-fadeIn">
      <div className="text-center max-w-md">
        <div className="text-6xl mb-4">{icon}</div>
        <h3 className="text-xl font-semibold text-gray-900 mb-2">{title}</h3>
        <p className="text-sm text-gray-600 mb-6">{description}</p>
        {action && (
          <button
            onClick={action.onClick}
            className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium btn-press"
          >
            {action.label}
          </button>
        )}
      </div>
    </div>
  )
}
