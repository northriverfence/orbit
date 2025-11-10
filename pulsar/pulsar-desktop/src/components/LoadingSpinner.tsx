interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg' | 'xl'
  color?: 'primary' | 'white' | 'gray'
  className?: string
}

export default function LoadingSpinner({
  size = 'md',
  color = 'primary',
  className = ''
}: LoadingSpinnerProps) {
  const sizeClasses = {
    sm: 'w-4 h-4 border-2',
    md: 'w-8 h-8 border-2',
    lg: 'w-12 h-12 border-3',
    xl: 'w-16 h-16 border-4',
  }

  const colorClasses = {
    primary: 'border-blue-600 border-t-transparent',
    white: 'border-white border-t-transparent',
    gray: 'border-gray-400 border-t-transparent',
  }

  return (
    <div
      className={`${sizeClasses[size]} ${colorClasses[color]} border-solid rounded-full animate-spin ${className}`}
      role="status"
      aria-label="Loading"
    />
  )
}
