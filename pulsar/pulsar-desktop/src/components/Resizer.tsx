import { useCallback, useEffect, useRef, useState } from 'react'

interface ResizerProps {
  direction: 'horizontal' | 'vertical'
  onResize: (delta: number) => void
  className?: string
}

export default function Resizer({ direction, onResize, className = '' }: ResizerProps) {
  const [isDragging, setIsDragging] = useState(false)
  const startPosRef = useRef<number>(0)

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()
      setIsDragging(true)
      startPosRef.current = direction === 'horizontal' ? e.clientY : e.clientX
    },
    [direction]
  )

  useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      const currentPos = direction === 'horizontal' ? e.clientY : e.clientX
      const delta = currentPos - startPosRef.current
      startPosRef.current = currentPos
      onResize(delta)
    }

    const handleMouseUp = () => {
      setIsDragging(false)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, direction, onResize])

  const cursorClass = direction === 'horizontal' ? 'cursor-ns-resize' : 'cursor-ew-resize'
  const sizeClass = direction === 'horizontal' ? 'h-1 w-full' : 'w-1 h-full'
  const hoverClass = direction === 'horizontal' ? 'hover:h-2' : 'hover:w-2'
  const activeClass = isDragging
    ? direction === 'horizontal'
      ? 'h-2 bg-blue-500'
      : 'w-2 bg-blue-500'
    : 'bg-gray-300'

  return (
    <div
      className={`${cursorClass} ${sizeClass} ${hoverClass} ${activeClass} transition-all duration-150 flex-shrink-0 ${className}`}
      onMouseDown={handleMouseDown}
      style={{
        userSelect: 'none',
        touchAction: 'none',
      }}
    >
      {/* Visual indicator */}
      <div
        className={`w-full h-full flex items-center justify-center ${
          isDragging ? 'bg-blue-500' : 'hover:bg-gray-400'
        }`}
      >
        {direction === 'horizontal' ? (
          <div className="w-8 h-0.5 bg-gray-500 rounded" />
        ) : (
          <div className="h-8 w-0.5 bg-gray-500 rounded" />
        )}
      </div>
    </div>
  )
}
