import LoadingSpinner from './LoadingSpinner'

interface InlineLoaderProps {
  message?: string
  size?: 'sm' | 'md'
}

export default function InlineLoader({ message, size = 'sm' }: InlineLoaderProps) {
  return (
    <div className="flex items-center gap-2 text-gray-600">
      <LoadingSpinner size={size} color="gray" />
      {message && <span className="text-sm">{message}</span>}
    </div>
  )
}
