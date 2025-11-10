import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import EmptyState from './EmptyState'

describe('EmptyState', () => {
  it('should render title and description', () => {
    render(
      <EmptyState
        title="No Items Found"
        description="There are no items to display"
      />
    )

    expect(screen.getByText('No Items Found')).toBeInTheDocument()
    expect(screen.getByText('There are no items to display')).toBeInTheDocument()
  })

  it('should render default icon', () => {
    render(
      <EmptyState
        title="Empty"
        description="No content"
      />
    )

    expect(screen.getByText('📭')).toBeInTheDocument()
  })

  it('should render custom icon', () => {
    render(
      <EmptyState
        icon="🎯"
        title="Empty"
        description="No content"
      />
    )

    expect(screen.getByText('🎯')).toBeInTheDocument()
    expect(screen.queryByText('📭')).not.toBeInTheDocument()
  })

  it('should not render action button when action is not provided', () => {
    render(
      <EmptyState
        title="Empty"
        description="No content"
      />
    )

    const button = screen.queryByRole('button')
    expect(button).not.toBeInTheDocument()
  })

  it('should render action button when action is provided', () => {
    const action = {
      label: 'Add Item',
      onClick: vi.fn(),
    }

    render(
      <EmptyState
        title="Empty"
        description="No content"
        action={action}
      />
    )

    expect(screen.getByText('Add Item')).toBeInTheDocument()
  })

  it('should call action onClick when button is clicked', () => {
    const onClick = vi.fn()
    const action = {
      label: 'Create New',
      onClick,
    }

    render(
      <EmptyState
        title="Empty"
        description="No content"
        action={action}
      />
    )

    const button = screen.getByText('Create New')
    fireEvent.click(button)

    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('should have fade-in animation', () => {
    const { container } = render(
      <EmptyState
        title="Empty"
        description="No content"
      />
    )

    const emptyState = container.firstChild as HTMLElement
    expect(emptyState.className).toContain('animate-fadeIn')
  })

  it('should center content', () => {
    const { container } = render(
      <EmptyState
        title="Empty"
        description="No content"
      />
    )

    const emptyState = container.firstChild as HTMLElement
    expect(emptyState.className).toContain('flex')
    expect(emptyState.className).toContain('items-center')
    expect(emptyState.className).toContain('justify-center')
  })

  it('should render title with proper styling', () => {
    render(
      <EmptyState
        title="Test Title"
        description="Test description"
      />
    )

    const title = screen.getByText('Test Title')
    expect(title.className).toContain('text-xl')
    expect(title.className).toContain('font-semibold')
  })

  it('should render description with proper styling', () => {
    render(
      <EmptyState
        title="Title"
        description="Test description"
      />
    )

    const description = screen.getByText('Test description')
    expect(description.className).toContain('text-sm')
    expect(description.className).toContain('text-gray-600')
  })

  it('should render large icon', () => {
    const { container } = render(
      <EmptyState
        icon="🚀"
        title="Title"
        description="Description"
      />
    )

    const iconContainer = screen.getByText('🚀')
    expect(iconContainer.className).toContain('text-6xl')
  })

  it('should render action button with primary styling', () => {
    const action = {
      label: 'Take Action',
      onClick: vi.fn(),
    }

    render(
      <EmptyState
        title="Title"
        description="Description"
        action={action}
      />
    )

    const button = screen.getByText('Take Action')
    expect(button.className).toContain('bg-blue-600')
    expect(button.className).toContain('text-white')
  })

  it('should have hover effect on action button', () => {
    const action = {
      label: 'Click Me',
      onClick: vi.fn(),
    }

    render(
      <EmptyState
        title="Title"
        description="Description"
        action={action}
      />
    )

    const button = screen.getByText('Click Me')
    expect(button.className).toContain('hover:bg-blue-700')
  })

  it('should constrain content width', () => {
    const { container } = render(
      <EmptyState
        title="Title"
        description="Description"
      />
    )

    const contentContainer = container.querySelector('.max-w-md')
    expect(contentContainer).toBeInTheDocument()
  })

  it('should render all elements in correct order', () => {
    const action = {
      label: 'Action',
      onClick: vi.fn(),
    }

    const { container } = render(
      <EmptyState
        icon="🎨"
        title="Empty State"
        description="No items here"
        action={action}
      />
    )

    const elements = container.querySelectorAll('.text-center > *')
    expect(elements.length).toBe(4) // icon, title, description, button
  })
})
