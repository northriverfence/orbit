/**
 * Accessibility Utilities
 *
 * Hooks and utilities for improved accessibility in React components
 */

import { useEffect, useRef, useCallback } from 'react';

/**
 * Hook to trap focus within a modal dialog
 * @param isOpen - Whether the modal is currently open
 * @example
 * const dialogRef = useFocusTrap(isOpen);
 * return <div ref={dialogRef}>...</div>
 */
export function useFocusTrap(isOpen: boolean) {
  const elementRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen || !elementRef.current) return;

    const element = elementRef.current;
    const focusableElements = element.querySelectorAll(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]):not([disabled])'
    );

    const firstFocusable = focusableElements[0] as HTMLElement;
    const lastFocusable = focusableElements[focusableElements.length - 1] as HTMLElement;

    // Store previously focused element
    const previouslyFocused = document.activeElement as HTMLElement;

    // Focus first element
    if (firstFocusable) {
      firstFocusable.focus();
    }

    // Handle tab key
    const handleTab = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable?.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable?.focus();
        }
      }
    };

    element.addEventListener('keydown', handleTab);

    // Cleanup
    return () => {
      element.removeEventListener('keydown', handleTab);
      // Restore focus
      if (previouslyFocused) {
        previouslyFocused.focus();
      }
    };
  }, [isOpen]);

  return elementRef;
}

/**
 * Hook to handle Escape key press
 * @param callback - Function to call when Escape is pressed
 * @param isEnabled - Whether the handler is enabled
 * @example
 * useEscapeKey(() => setModalOpen(false), isModalOpen);
 */
export function useEscapeKey(callback: () => void, isEnabled: boolean = true) {
  useEffect(() => {
    if (!isEnabled) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        callback();
      }
    };

    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('keydown', handleEscape);
    };
  }, [callback, isEnabled]);
}

/**
 * Hook to announce screen reader messages
 * @returns Function to announce a message
 * @example
 * const announce = useScreenReaderAnnouncement();
 * announce('Item added to cart');
 */
export function useScreenReaderAnnouncement() {
  const announcerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    // Create announcer element if it doesn't exist
    if (!announcerRef.current) {
      const announcer = document.createElement('div');
      announcer.setAttribute('role', 'status');
      announcer.setAttribute('aria-live', 'polite');
      announcer.setAttribute('aria-atomic', 'true');
      announcer.className = 'sr-only';
      announcer.style.position = 'absolute';
      announcer.style.left = '-10000px';
      announcer.style.width = '1px';
      announcer.style.height = '1px';
      announcer.style.overflow = 'hidden';
      document.body.appendChild(announcer);
      announcerRef.current = announcer;
    }

    return () => {
      if (announcerRef.current) {
        document.body.removeChild(announcerRef.current);
      }
    };
  }, []);

  const announce = useCallback((message: string) => {
    if (announcerRef.current) {
      announcerRef.current.textContent = message;
    }
  }, []);

  return announce;
}

/**
 * Hook to manage keyboard navigation in a list
 * @param itemCount - Number of items in the list
 * @param onSelect - Callback when an item is selected (Enter/Space)
 * @returns Current focused index and key handler
 * @example
 * const { focusedIndex, handleKeyDown } = useListKeyboardNavigation(items.length, handleSelect);
 */
export function useListKeyboardNavigation(
  itemCount: number,
  onSelect?: (index: number) => void
) {
  const [focusedIndex, setFocusedIndex] = React.useState(0);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setFocusedIndex((prev) => (prev + 1) % itemCount);
          break;

        case 'ArrowUp':
          event.preventDefault();
          setFocusedIndex((prev) => (prev - 1 + itemCount) % itemCount);
          break;

        case 'Home':
          event.preventDefault();
          setFocusedIndex(0);
          break;

        case 'End':
          event.preventDefault();
          setFocusedIndex(itemCount - 1);
          break;

        case 'Enter':
        case ' ':
          event.preventDefault();
          if (onSelect) {
            onSelect(focusedIndex);
          }
          break;
      }
    },
    [itemCount, focusedIndex, onSelect]
  );

  return { focusedIndex, handleKeyDown, setFocusedIndex };
}

/**
 * Generates a unique ID for accessibility attributes
 * @param prefix - Prefix for the ID
 * @returns Unique ID
 */
let idCounter = 0;
export function useUniqueId(prefix: string = 'id'): string {
  const idRef = useRef<string | null>(null);

  if (idRef.current === null) {
    idRef.current = `${prefix}-${idCounter++}`;
  }

  return idRef.current;
}

/**
 * Props for accessible modal dialogs
 */
export interface AccessibleModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
}

/**
 * Hook combining all modal accessibility features
 * @param props - Modal properties
 * @returns Refs and handlers for accessibility
 */
export function useAccessibleModal(props: AccessibleModalProps) {
  const { isOpen, onClose, title, description } = props;
  const dialogRef = useFocusTrap(isOpen);
  const titleId = useUniqueId('dialog-title');
  const descriptionId = useUniqueId('dialog-description');
  const announce = useScreenReaderAnnouncement();

  useEscapeKey(onClose, isOpen);

  // Announce when modal opens
  useEffect(() => {
    if (isOpen) {
      announce(`${title} dialog opened`);
    }
  }, [isOpen, title, announce]);

  return {
    dialogRef,
    titleId,
    descriptionId,
    ariaProps: {
      role: 'dialog',
      'aria-modal': 'true' as const,
      'aria-labelledby': titleId,
      'aria-describedby': description ? descriptionId : undefined,
    },
  };
}

// Re-export React for the hook
import React from 'react';
