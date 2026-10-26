/**
 * Lightweight, modular controller dispatcher for data-* attributes.
 * Works seamlessly with HTMX server-rendered DOM swaps.
 */
(() => {
  const initializedElements = new WeakSet();

  function setThemeCookie(theme) {
    const maxAge = 365 * 24 * 60 * 60; // 1 year
    document.cookie = `theme=${encodeURIComponent(theme)}; path=/; max-age=${maxAge}; SameSite=Lax`;
  }

  // Registry of modular controllers
  const controllers = {
    toast: (el) => {
      // Auto-dismissible toast notification
      const duration = parseInt(el.getAttribute('data-duration') || '4500', 10);
      let timeoutId = null;

      const dismiss = () => {
        el.classList.add('opacity-0', 'transition-all', 'duration-300', 'translate-x-4');
        setTimeout(() => {
          el.remove();
        }, 300);
      };

      const startTimer = () => {
        if (duration > 0 && !timeoutId) {
          timeoutId = setTimeout(dismiss, duration);
        }
      };

      const clearTimer = () => {
        if (timeoutId) {
          clearTimeout(timeoutId);
          timeoutId = null;
        }
      };

      // Start auto dismiss
      startTimer();

      // Pause timer when hovering so user can read / click
      el.addEventListener('mouseenter', clearTimer);
      el.addEventListener('mouseleave', startTimer);

      // Manual dismiss button
      const closeBtn = el.querySelector('[data-action="dismiss"]');
      if (closeBtn) {
        closeBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          clearTimer();
          dismiss();
        });
      }
    },

    theme: (el) => {
      // Toggles dark / light theme and saves to cookie & localStorage
      el.addEventListener('click', () => {
        const isDark = document.documentElement.classList.toggle('dark');
        const theme = isDark ? 'dark' : 'light';
        setThemeCookie(theme);
        try {
          localStorage.setItem('theme', theme);
        } catch (_) {}
      });
    },

    sidebar: (el) => {
      // Controls the responsive sidebar
      const sidebar = document.getElementById('app-sidebar');
      const backdrop = document.getElementById('sidebar-backdrop');
      if (!sidebar) return;

      const isLargeScreen = () => window.matchMedia('(min-width: 1024px)').matches;

      el.addEventListener('click', (e) => {
        e.stopPropagation();
        if (isLargeScreen()) {
          sidebar.classList.remove('sidebar-mobile-open');
          sidebar.classList.add('lg:translate-x-0');
          sidebar.classList.toggle('sidebar-collapsed');
        } else {
          sidebar.classList.remove('sidebar-collapsed');
          const willOpen = !sidebar.classList.contains('sidebar-mobile-open');
          sidebar.classList.toggle('sidebar-mobile-open', willOpen);
          if (willOpen) {
            sidebar.classList.remove('-translate-x-full');
            sidebar.classList.add('translate-x-0');
          } else {
            sidebar.classList.remove('translate-x-0');
            sidebar.classList.add('-translate-x-full');
          }
          if (backdrop) {
            backdrop.classList.toggle('hidden', !willOpen);
          }
        }
      });

      // Close sidebar when navigating on mobile
      const navLinks = sidebar.querySelectorAll('nav a');
      navLinks.forEach((link) => {
        link.addEventListener('click', () => {
          if (!isLargeScreen()) {
            sidebar.classList.remove('sidebar-mobile-open');
            sidebar.classList.remove('translate-x-0');
            sidebar.classList.add('-translate-x-full');
            if (backdrop) backdrop.classList.add('hidden');
          }
        });
      });

      // Sync state when resizing across mobile / desktop breakpoint
      window.addEventListener('resize', () => {
        if (isLargeScreen()) {
          sidebar.classList.remove('sidebar-mobile-open');
          sidebar.classList.remove('translate-x-0');
          sidebar.classList.remove('-translate-x-full');
          if (backdrop) backdrop.classList.add('hidden');
        } else {
          sidebar.classList.remove('sidebar-collapsed');
          if (!sidebar.classList.contains('sidebar-mobile-open')) {
            sidebar.classList.add('-translate-x-full');
            sidebar.classList.remove('translate-x-0');
          }
        }
      });
    },

    'custom-select': (root) => {
      // Accessible custom Select & Combobox with HTMX search, keyboard navigation, and hidden input synchronization
      const trigger = root.querySelector('[data-select-trigger]');
      const popover = root.querySelector('[data-select-popover]');
      const listbox = root.querySelector('[data-select-listbox]');
      const searchInput = root.querySelector('[data-select-search]');
      const hiddenInput = root.querySelector('input[type="hidden"]');
      const labelEl = root.querySelector('[data-select-label]');
      const chevron = root.querySelector('[data-select-chevron]');

      if (!trigger || !popover || !listbox || !hiddenInput) return;

      let activeIndex = -1;

      function getOptions() {
        return Array.from(listbox.querySelectorAll('[role="option"]:not([data-disabled="true"])'));
      }

      function isOpen() {
        return !popover.classList.contains('hidden');
      }

      function openDropdown() {
        // Close any other open custom-selects first
        document.querySelectorAll('[data-controller="custom-select"]').forEach((other) => {
          if (other !== root) {
            const p = other.querySelector('[data-select-popover]');
            const t = other.querySelector('[data-select-trigger]');
            const c = other.querySelector('[data-select-chevron]');
            if (p) p.classList.add('hidden');
            if (t) t.setAttribute('aria-expanded', 'false');
            if (c) c.classList.remove('rotate-180');
            other.classList.remove('z-30');
          }
        });

        popover.classList.remove('hidden');
        trigger.setAttribute('aria-expanded', 'true');
        if (chevron) chevron.classList.add('rotate-180');
        root.classList.add('z-30');

        // Highlight currently selected item or first option
        const options = getOptions();
        const currentVal = hiddenInput.value;
        activeIndex = options.findIndex((opt) => opt.getAttribute('data-value') === currentVal);
        if (activeIndex === -1 && options.length > 0) activeIndex = 0;
        highlightOption(activeIndex);

        if (searchInput) {
          if (searchInput.value !== '') {
            searchInput.value = '';
            if (searchInput.hasAttribute('hx-get')) {
              if (window.htmx) window.htmx.trigger(searchInput, 'search');
            } else {
              filterLocalOptions('');
            }
          } else if (!searchInput.hasAttribute('hx-get')) {
            filterLocalOptions('');
          }
          requestAnimationFrame(() => searchInput.focus());
        }
      }

      function closeDropdown(refocus = true) {
        if (!isOpen()) return;
        popover.classList.add('hidden');
        trigger.setAttribute('aria-expanded', 'false');
        trigger.removeAttribute('aria-activedescendant');
        if (chevron) chevron.classList.remove('rotate-180');
        root.classList.remove('z-30');
        activeIndex = -1;
        if (refocus) trigger.focus();
      }

      function highlightOption(index) {
        const options = getOptions();
        options.forEach((opt, idx) => {
          if (idx === index) {
            opt.classList.add('bg-accent', 'text-accent-foreground');
            trigger.setAttribute('aria-activedescendant', opt.id || '');
            opt.scrollIntoView({ block: 'nearest' });
          } else {
            opt.classList.remove('bg-accent', 'text-accent-foreground');
          }
        });
        activeIndex = index;
      }

      function selectOption(optionEl) {
        if (!optionEl || optionEl.getAttribute('data-disabled') === 'true') return;

        const val = optionEl.getAttribute('data-value') || '';
        const label = optionEl.getAttribute('data-label') || optionEl.textContent.trim();

        // 1. Update visual label
        if (labelEl) {
          labelEl.textContent = label;
          labelEl.classList.remove('text-muted-foreground');
          labelEl.classList.add('text-foreground');
        }

        // 2. Mark aria-selected and checkmarks
        listbox.querySelectorAll('[role="option"]').forEach((opt) => {
          const isSelected = opt === optionEl;
          opt.setAttribute('aria-selected', isSelected ? 'true' : 'false');
          const check = opt.querySelector('[data-select-check]');
          if (check) {
            check.classList.toggle('opacity-0', !isSelected);
            check.classList.toggle('opacity-100', isSelected);
          }
        });

        // 3. Update hidden input and dispatch standard bubbling events for HTMX
        if (hiddenInput.value !== val) {
          hiddenInput.value = val;
          hiddenInput.dispatchEvent(new Event('input', { bubbles: true }));
          hiddenInput.dispatchEvent(new Event('change', { bubbles: true }));
        }

        closeDropdown(true);
      }

      function filterLocalOptions(query) {
        const q = query.trim().toLowerCase();
        const allOptions = listbox.querySelectorAll('[role="option"]');
        let visibleCount = 0;

        allOptions.forEach((opt) => {
          const text = (opt.getAttribute('data-label') || opt.textContent).toLowerCase();
          const matches = !q || text.includes(q);
          opt.classList.toggle('hidden', !matches);
          if (matches) visibleCount++;
        });

        const emptyMsg = listbox.querySelector('[data-select-empty]');
        if (emptyMsg) {
          emptyMsg.classList.toggle('hidden', visibleCount > 0);
        }

        activeIndex = 0;
        highlightOption(0);
      }

      // --- Event Listeners ---

      // 1. Trigger click toggle
      trigger.addEventListener('click', (e) => {
        e.preventDefault();
        isOpen() ? closeDropdown(false) : openDropdown();
      });

      // 2. Option selection via Event Delegation (handles dynamic HTMX swaps cleanly)
      listbox.addEventListener('click', (e) => {
        const option = e.target.closest('[role="option"]');
        if (option) {
          e.preventDefault();
          selectOption(option);
        }
      });

      // 3. Local or Remote Search Handling
      if (searchInput && !searchInput.hasAttribute('hx-get')) {
        searchInput.addEventListener('input', (e) => {
          filterLocalOptions(e.target.value);
        });
      }

      // Allow global mutation events to re-trigger remote search if needed
      root.addEventListener('custom-select:clear-cache', () => {
        if (searchInput && searchInput.hasAttribute('hx-get') && window.htmx) {
          window.htmx.trigger(searchInput, 'search');
        }
      });

      // 4. Keyboard Navigation on root / search input / trigger
      const handleKeydown = (e) => {
        if (!isOpen()) {
          if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            openDropdown();
          }
          return;
        }

        const options = getOptions();

        switch (e.key) {
          case 'ArrowDown':
            e.preventDefault();
            if (options.length === 0) return;
            highlightOption(activeIndex + 1 >= options.length ? 0 : activeIndex + 1);
            break;

          case 'ArrowUp':
            e.preventDefault();
            if (options.length === 0) return;
            highlightOption(activeIndex - 1 < 0 ? options.length - 1 : activeIndex - 1);
            break;

          case 'Enter':
            e.preventDefault(); // Stop form submission
            if (activeIndex >= 0 && options[activeIndex]) {
              selectOption(options[activeIndex]);
            }
            break;

          case 'Escape':
            e.preventDefault();
            e.stopPropagation(); // Stop parent modal dismissal on first Escape
            closeDropdown(true);
            break;

          case 'Tab':
            closeDropdown(false);
            break;
        }
      };

      root.addEventListener('keydown', handleKeydown);

      // 5. Reset active highlight when HTMX swaps new options into listbox
      listbox.addEventListener('htmx:afterSwap', () => {
        activeIndex = 0;
        highlightOption(0);
      });
    }
  };

  /**
   * Central delegated document click listener for outside clicks, modal dismissal, and mobile drawer dismissal.
   */
  document.addEventListener('click', (e) => {
    // 1. Outside click dismissal for custom-select
    document.querySelectorAll('[data-controller="custom-select"]').forEach((s) => {
      if (!s.contains(e.target)) {
        const popover = s.querySelector('[data-select-popover]');
        const trigger = s.querySelector('[data-select-trigger]');
        const chevron = s.querySelector('[data-select-chevron]');
        if (popover && !popover.classList.contains('hidden')) {
          popover.classList.add('hidden');
          if (trigger) {
            trigger.setAttribute('aria-expanded', 'false');
            trigger.removeAttribute('aria-activedescendant');
          }
          if (chevron) chevron.classList.remove('rotate-180');
          s.classList.remove('z-30');
        }
      }
    });

    // 2. Dismiss modal on backdrop or data-action="close-modal"
    if (e.target.matches('[data-action="close-modal"]') || e.target.closest('[data-action="close-modal"]') || e.target.id === 'modal-backdrop' || e.target.id === 'drawer-backdrop') {
      const modalContainer = document.getElementById('modal-container');
      if (modalContainer && (e.target.closest('#modal-container') || e.target.id === 'modal-backdrop')) {
        modalContainer.innerHTML = '';
      }
      const drawerContainer = document.getElementById('drawer-container');
      if (drawerContainer && (e.target.closest('#drawer-container') || e.target.id === 'drawer-backdrop')) {
        drawerContainer.innerHTML = '';
      }
    }

    // 3. Dismiss mobile sidebar on backdrop or data-action="close-sidebar"
    if (e.target.matches('[data-action="close-sidebar"]') || e.target.closest('[data-action="close-sidebar"]') || e.target.id === 'sidebar-backdrop') {
      const sidebar = document.getElementById('app-sidebar');
      const backdrop = document.getElementById('sidebar-backdrop');
      if (sidebar) {
        sidebar.classList.remove('sidebar-mobile-open', 'translate-x-0');
        sidebar.classList.add('-translate-x-full');
      }
      if (backdrop) {
        backdrop.classList.add('hidden');
      }
    }
  });

  /**
   * Global Escape key listener to dismiss open modals, drawers, and selects
   */
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      const modalContainer = document.getElementById('modal-container');
      if (modalContainer && modalContainer.innerHTML.trim() !== '') {
        modalContainer.innerHTML = '';
        return;
      }
      const drawerContainer = document.getElementById('drawer-container');
      if (drawerContainer && drawerContainer.innerHTML.trim() !== '') {
        drawerContainer.innerHTML = '';
        return;
      }
      const sidebar = document.getElementById('app-sidebar');
      const backdrop = document.getElementById('sidebar-backdrop');
      if (sidebar && sidebar.classList.contains('sidebar-mobile-open')) {
        sidebar.classList.remove('sidebar-mobile-open', 'translate-x-0');
        sidebar.classList.add('-translate-x-full');
        if (backdrop) backdrop.classList.add('hidden');
      }
    }
  });

  /**
   * Scans a root container (document or HTMX swapped node)
   * and initializes any registered data-controller elements.
   */
  function initControllers(root = document) {
    if (!root.querySelectorAll) return;

    // Scan for all elements declaring data-controller
    const elements = root.querySelectorAll('[data-controller]');
    elements.forEach((el) => {
      if (initializedElements.has(el)) return;

      const name = el.getAttribute('data-controller');
      const initFn = controllers[name];

      if (typeof initFn === 'function') {
        try {
          initFn(el);
          initializedElements.add(el);
        } catch (err) {
          console.error(`[controller:${name}] initialization error:`, err);
        }
      }
    });

    // Also check root itself if it has a controller
    if (root.hasAttribute && root.hasAttribute('data-controller') && !initializedElements.has(root)) {
      const name = root.getAttribute('data-controller');
      const initFn = controllers[name];
      if (typeof initFn === 'function') {
        initFn(root);
        initializedElements.add(root);
      }
    }
  }

  // 1. Initial DOM load
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => initControllers(document));
  } else {
    initControllers(document);
  }

  // 2. Automatically re-bind whenever HTMX swaps new content into the DOM
  document.addEventListener('htmx:afterSwap', (evt) => {
    initControllers(evt.detail.target);
  });

  // 3. Coordinate boosted navigation: update active navigation link styling
  document.addEventListener('htmx:afterSettle', () => {
    const currentPath = window.location.pathname;
    const links = document.querySelectorAll('#app-sidebar nav a[href]');
    links.forEach((link) => {
      const href = link.getAttribute('href');
      const isActive = href === currentPath || (href !== '/' && currentPath.startsWith(href));
      if (isActive) {
        link.className = 'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium bg-primary text-primary-foreground shadow-sm transition-colors';
      } else {
        link.className = 'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors';
      }
    });
  });

  // 4. Clean up any open modal dialogs or drawers when navigating across main views
  document.addEventListener('htmx:beforeSwap', (e) => {
    if (e.detail && e.detail.target && e.detail.target.id === 'main-content') {
      const modal = document.getElementById('modal-container');
      if (modal) modal.innerHTML = '';
      const drawer = document.getElementById('drawer-container');
      if (drawer) drawer.innerHTML = '';
    }
  });

  // 5. Sanitize DOM prior to HTMX History Snapshots
  document.addEventListener('htmx:beforeHistorySave', () => {
    const modal = document.getElementById('modal-container');
    if (modal) modal.innerHTML = '';
    const drawer = document.getElementById('drawer-container');
    if (drawer) drawer.innerHTML = '';
    const toasts = document.getElementById('toast-container');
    if (toasts) toasts.innerHTML = '';

    document.querySelectorAll('[data-controller="custom-select"]').forEach((select) => {
      const popover = select.querySelector('[data-select-popover]');
      const trigger = select.querySelector('[data-select-trigger]');
      const chevron = select.querySelector('[data-select-chevron]');
      if (popover) popover.classList.add('hidden');
      if (trigger) {
        trigger.setAttribute('aria-expanded', 'false');
        trigger.removeAttribute('aria-activedescendant');
      }
      if (chevron) chevron.classList.remove('rotate-180');
      select.classList.remove('z-30');
    });
  });

  // 6. Reactive Cache Invalidation on Mutations
  document.body.addEventListener('library:data-changed', () => {
    try {
      sessionStorage.removeItem('htmx-history-cache');
    } catch (_) {}

    document.querySelectorAll('[data-controller="custom-select"]').forEach((el) => {
      el.dispatchEvent(new CustomEvent('custom-select:clear-cache'));
    });
  });

  // Expose registration hook for adding new controllers dynamically
  window.registerController = (name, fn) => {
    controllers[name] = fn;
    initControllers(document);
  };
})();
