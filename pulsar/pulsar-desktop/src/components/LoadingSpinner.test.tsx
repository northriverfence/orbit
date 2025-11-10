import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'
import LoadingSpinner from './LoadingSpinner'

describe('LoadingSpinner', () => {
  it('should render with default props', () => {
    const { container } = render(<LoadingSpinner />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner).toHaveAttribute('role', 'status')
    expect(spinner).toHaveAttribute('aria-label', 'Loading')
  })

  it('should render with small size', () => {
    const { container } = render(<LoadingSpinner size="sm" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('w-4')
    expect(spinner.className).toContain('h-4')
  })

  it('should render with medium size (default)', () => {
    const { container } = render(<LoadingSpinner size="md" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('w-8')
    expect(spinner.className).toContain('h-8')
  })

  it('should render with large size', () => {
    const { container } = render(<LoadingSpinner size="lg" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('w-12')
    expect(spinner.className).toContain('h-12')
  })

  it('should render with extra large size', () => {
    const { container } = render(<LoadingSpinner size="xl" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('w-16')
    expect(spinner.className).toContain('h-16')
  })

  it('should render with primary color (default)', () => {
    const { container } = render(<LoadingSpinner color="primary" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('border-blue-600')
  })

  it('should render with white color', () => {
    const { container } = render(<LoadingSpinner color="white" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('border-white')
  })

  it('should render with gray color', () => {
    const { container } = render(<LoadingSpinner color="gray" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('border-gray-400')
  })

  it('should have spin animation', () => {
    const { container } = render(<LoadingSpinner />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('animate-spin')
  })

  it('should be rounded', () => {
    const { container } = render(<LoadingSpinner />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('rounded-full')
  })

  it('should accept custom className', () => {
    const { container } = render(<LoadingSpinner className="custom-class" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('custom-class')
  })

  it('should combine size and color props', () => {
    const { container } = render(<LoadingSpinner size="lg" color="white" />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('w-12')
    expect(spinner.className).toContain('border-white')
  })

  it('should have transparent top border for spin effect', () => {
    const { container } = render(<LoadingSpinner />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner.className).toContain('border-t-transparent')
  })

  it('should have proper accessibility attributes', () => {
    const { container } = render(<LoadingSpinner />)

    const spinner = container.firstChild as HTMLElement
    expect(spinner).toHaveAttribute('role', 'status')
    expect(spinner).toHaveAttribute('aria-label', 'Loading')
  })
})
