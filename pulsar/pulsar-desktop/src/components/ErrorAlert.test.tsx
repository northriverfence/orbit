import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ErrorAlert from './ErrorAlert'

describe('ErrorAlert', () => {
  it('should render error message', () => {
    render(<ErrorAlert message="Something went wrong" />)

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })

  it('should render with title', () => {
    render(<ErrorAlert title="Error Title" message="Error message" />)

    expect(screen.getByText('Error Title')).toBeInTheDocument()
    expect(screen.getByText('Error message')).toBeInTheDocument()
  })

  it('should render without title', () => {
    render(<ErrorAlert message="Error message" />)

    expect(screen.getByText('Error message')).toBeInTheDocument()
  })

  it('should render error type with red styling', () => {
    const { container } = render(<ErrorAlert message="Error" type="error" />)

    const alert = container.firstChild as HTMLElement
    expect(alert.className).toContain('bg-red-50')
    expect(alert.className).toContain('border-red-200')
  })

  it('should render warning type with yellow styling', () => {
    const { container } = render(<ErrorAlert message="Warning" type="warning" />)

    const alert = container.firstChild as HTMLElement
    expect(alert.className).toContain('bg-yellow-50')
    expect(alert.className).toContain('border-yellow-200')
  })

  it('should default to error type', () => {
    const { container } = render(<ErrorAlert message="Test" />)

    const alert = container.firstChild as HTMLElement
    expect(alert.className).toContain('bg-red-50')
  })

  it('should render details section when provided', () => {
    render(
      <ErrorAlert
        message="Error occurred"
        details="Stack trace: Error at line 42"
      />
    )

    expect(screen.getByText('Show details')).toBeInTheDocument()
  })

  it('should toggle details visibility', () => {
    render(
      <ErrorAlert
        message="Error occurred"
        details="Detailed error information"
      />
    )

    const summary = screen.getByText('Show details')
    fireEvent.click(summary)

    expect(screen.getByText('Detailed error information')).toBeInTheDocument()
  })

  it('should not render details section when not provided', () => {
    render(<ErrorAlert message="Error" />)

    expect(screen.queryByText('Show details')).not.toBeInTheDocument()
  })

  it('should render Try Again button when onRetry is provided', () => {
    const onRetry = vi.fn()
    render(<ErrorAlert message="Error" onRetry={onRetry} />)

    expect(screen.getByText('Try Again')).toBeInTheDocument()
  })

  it('should call onRetry when Try Again button is clicked', () => {
    const onRetry = vi.fn()
    render(<ErrorAlert message="Error" onRetry={onRetry} />)

    const button = screen.getByText('Try Again')
    fireEvent.click(button)

    expect(onRetry).toHaveBeenCalledTimes(1)
  })

  it('should render Dismiss button when onDismiss is provided', () => {
    const onDismiss = vi.fn()
    render(<ErrorAlert message="Error" onDismiss={onDismiss} />)

    expect(screen.getByText('Dismiss')).toBeInTheDocument()
  })

  it('should call onDismiss when Dismiss button is clicked', () => {
    const onDismiss = vi.fn()
    render(<ErrorAlert message="Error" onDismiss={onDismiss} />)

    const button = screen.getByText('Dismiss')
    fireEvent.click(button)

    expect(onDismiss).toHaveBeenCalledTimes(1)
  })

  it('should render close X button when onDismiss is provided', () => {
    const onDismiss = vi.fn()
    const { container } = render(<ErrorAlert message="Error" onDismiss={onDismiss} />)

    // Find the X close button (separate from Dismiss button)
    const closeButtons = container.querySelectorAll('button')
    expect(closeButtons.length).toBeGreaterThan(1)
  })

  it('should call onDismiss when close X button is clicked', () => {
    const onDismiss = vi.fn()
    const { container } = render(<ErrorAlert message="Error" onDismiss={onDismiss} />)

    // The X button is the last button
    const buttons = container.querySelectorAll('button')
    const xButton = buttons[buttons.length - 1]
    fireEvent.click(xButton)

    expect(onDismiss).toHaveBeenCalledTimes(1)
  })

  it('should render both Try Again and Dismiss buttons', () => {
    const onRetry = vi.fn()
    const onDismiss = vi.fn()
    render(<ErrorAlert message="Error" onRetry={onRetry} onDismiss={onDismiss} />)

    expect(screen.getByText('Try Again')).toBeInTheDocument()
    expect(screen.getByText('Dismiss')).toBeInTheDocument()
  })

  it('should not render action buttons when neither callback is provided', () => {
    const { container } = render(<ErrorAlert message="Error" />)

    const tryAgainButton = screen.queryByText('Try Again')
    const dismissButton = screen.queryByText('Dismiss')

    expect(tryAgainButton).not.toBeInTheDocument()
    expect(dismissButton).not.toBeInTheDocument()
  })

  it('should have slide-in animation', () => {
    const { container } = render(<ErrorAlert message="Error" />)

    const alert = container.firstChild as HTMLElement
    expect(alert.className).toContain('animate-slideInFromTop')
  })

  it('should render error icon for error type', () => {
    const { container } = render(<ErrorAlert message="Error" type="error" />)

    const svg = container.querySelector('svg')
    expect(svg).toBeInTheDocument()
  })

  it('should render warning icon for warning type', () => {
    const { container } = render(<ErrorAlert message="Warning" type="warning" />)

    const svg = container.querySelector('svg')
    expect(svg).toBeInTheDocument()
  })

  it('should render details in preformatted block', () => {
    const { container } = render(
      <ErrorAlert
        message="Error"
        details="Line 1\nLine 2\nLine 3"
      />
    )

    fireEvent.click(screen.getByText('Show details'))

    // Find the pre element
    const pre = container.querySelector('pre')
    expect(pre).toBeInTheDocument()
    expect(pre?.textContent).toContain('Line 1')
    expect(pre?.textContent).toContain('Line 2')
    expect(pre?.textContent).toContain('Line 3')
  })

  it('should combine title, message, and details', () => {
    render(
      <ErrorAlert
        title="Connection Failed"
        message="Unable to connect to server"
        details="Error: ECONNREFUSED"
      />
    )

    expect(screen.getByText('Connection Failed')).toBeInTheDocument()
    expect(screen.getByText('Unable to connect to server')).toBeInTheDocument()
    expect(screen.getByText('Show details')).toBeInTheDocument()
  })
})
