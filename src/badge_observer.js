/**
 * Wafy - WhatsApp Web Badge & Notification Injection Script
 *
 * Responsibilities:
 * 1. Monitor unread message counts via document.title and sidebar DOM badges.
 * 2. Debounce and transmit unread count updates via WebKit IPC.
 * 3. Intercept HTML5 Notification API and route to WebKit IPC for native Linux desktop notifications.
 */

(function () {
  'use strict';

  // Prevent multiple injections
  if (window.__WAFY_INJECTED__) {
    return;
  }
  window.__WAFY_INJECTED__ = true;

  /**
   * Safely dispatch IPC messages to Wafy host application.
   */
  function postToWafy(payload) {
    try {
      if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        if (payload.type === 'unread_count') {
          window.__TAURI__.core.invoke('update_unread_count', { count: payload.count }).catch(function(){});
        } else if (payload.type === 'notification') {
          window.__TAURI__.core.invoke('show_notification', { title: payload.title, body: payload.body }).catch(function(){});
        }
      }
      if (
        window.webkit &&
        window.webkit.messageHandlers &&
        window.webkit.messageHandlers.wafy
      ) {
        window.webkit.messageHandlers.wafy.postMessage(JSON.stringify(payload));
      }
    } catch (err) {
      console.warn('[Wafy] IPC postMessage error:', err);
    }
  }

  // =========================================================================
  // 1. HTML5 Notification API Interception
  // =========================================================================

  const OriginalNotification = window.Notification;

  function WafyNotification(title, options) {
    const opts = options || {};

    // Forward notification to WebKit message handler
    if (
      window.webkit &&
      window.webkit.messageHandlers &&
      window.webkit.messageHandlers.wafy
    ) {
      window.webkit.messageHandlers.wafy.postMessage(
        JSON.stringify({
          type: 'notification',
          title: title || 'WhatsApp',
          body: opts.body || '',
          icon: opts.icon || ''
        })
      );
    }

    this.title = title || 'WhatsApp';
    this.body = opts.body || '';
    this.icon = opts.icon || '';
    this.tag = opts.tag || '';
    this.data = opts.data || null;

    this.onclick = null;
    this.onshow = null;
    this.onerror = null;
    this.onclose = null;

    this.close = function () {
      if (typeof this.onclose === 'function') {
        this.onclose();
      }
    };

    this.addEventListener = function (type, listener) {
      if (typeof listener !== 'function') return;
      if (type === 'click') this.onclick = listener;
      else if (type === 'close') this.onclose = listener;
      else if (type === 'show') this.onshow = listener;
      else if (type === 'error') this.onerror = listener;
    };

    this.removeEventListener = function (type, listener) {
      if (type === 'click' && this.onclick === listener) this.onclick = null;
      else if (type === 'close' && this.onclose === listener) this.onclose = null;
      else if (type === 'show' && this.onshow === listener) this.onshow = null;
      else if (type === 'error' && this.onerror === listener) this.onerror = null;
    };

    this.dispatchEvent = function () {
      return true;
    };

    // Trigger onshow asynchronously to mimic native behavior
    setTimeout(() => {
      if (typeof this.onshow === 'function') {
        this.onshow();
      }
    }, 0);
  }

  WafyNotification.permission = 'granted';
  WafyNotification.requestPermission = function (callback) {
    const res = Promise.resolve('granted');
    if (typeof callback === 'function') {
      callback('granted');
    }
    return res;
  };
  WafyNotification.maxActions = 2;

  if (OriginalNotification) {
    WafyNotification.prototype = Object.create(OriginalNotification.prototype);
    WafyNotification.prototype.constructor = WafyNotification;
  }

  try {
    Object.defineProperty(window, 'Notification', {
      value: WafyNotification,
      writable: true,
      configurable: true
    });
  } catch (e) {
    window.Notification = WafyNotification;
  }

  // Intercept navigator.permissions.query for 'notifications'
  if (navigator.permissions && typeof navigator.permissions.query === 'function') {
    const originalQuery = navigator.permissions.query.bind(navigator.permissions);
    navigator.permissions.query = function (parameters) {
      if (parameters && parameters.name === 'notifications') {
        return Promise.resolve({
          state: 'granted',
          status: 'granted',
          onchange: null,
          addEventListener: function () {},
          removeEventListener: function () {},
          dispatchEvent: function () {
            return false;
          }
        });
      }
      return originalQuery(parameters);
    };
  }

  // =========================================================================
  // 2. Unread Badge Observer & Title Parser
  // =========================================================================

  let lastReportedCount = null;
  let debounceTimer = null;
  const DEBOUNCE_DELAY_MS = 250;

  /**
   * Parse unread message count from document.title.
   * WhatsApp Web formats titles as:
   *   "(N) WhatsApp"
   *   "(N) Chat Name"
   * Extract N as an integer. If no parentheses or count is 0, report 0.
   */
  function parseTitleUnreadCount(title) {
    if (!title || typeof title !== 'string') {
      return 0;
    }
    const match = title.match(/^\s*\(\s*(\d+)\+?\s*\)/);
    if (match && match[1]) {
      const parsed = parseInt(match[1], 10);
      return isNaN(parsed) || parsed <= 0 ? 0 : parsed;
    }
    return 0;
  }

  /**
   * Parse unread message badges from the WhatsApp DOM.
   * Inspects aria-labels and data-testids for unread badge counters.
   */
  function parseDomUnreadCount() {
    let count = 0;
    try {
      // Selectors matching WhatsApp Web's unread badges across versions
      const badgeElements = document.querySelectorAll(
        'span[aria-label*="unread" i],' +
        'span[aria-label*="message" i],' +
        '[data-testid="unread-count"],' +
        '[data-testid="icon-unread-count"],' +
        '[aria-label*="unread message" i],' +
        'div[aria-label*="unread" i]'
      );

      const countedElements = new Set();

      badgeElements.forEach((el) => {
        // Skip elements already checked or nested inside counted elements
        if (countedElements.has(el)) return;
        for (const counted of countedElements) {
          if (counted.contains(el) || el.contains(counted)) {
            return;
          }
        }
        countedElements.add(el);

        // 1. Try reading numerical content from the text inside badge
        const textContent = el.textContent ? el.textContent.trim() : '';
        const parsedText = parseInt(textContent, 10);
        if (!isNaN(parsedText) && parsedText > 0) {
          count += parsedText;
          return;
        }

        // 2. Try parsing number from aria-label (e.g. "5 unread messages")
        const ariaLabel = el.getAttribute('aria-label') || '';
        const match = ariaLabel.match(/(\d+)/);
        if (match && match[1]) {
          const parsedAria = parseInt(match[1], 10);
          if (!isNaN(parsedAria) && parsedAria > 0) {
            count += parsedAria;
            return;
          }
        }

        // 3. Fallback: unnumbered badge/dot indicates at least 1 unread message
        if (ariaLabel.toLowerCase().includes('unread')) {
          count += 1;
        }
      });
    } catch (err) {
      console.warn('[Wafy] DOM unread parse error:', err);
    }
    return count;
  }

  /**
   * Evaluate unread count from both title and DOM, and dispatch if changed.
   */
  function updateUnreadCount() {
    const titleCount = parseTitleUnreadCount(document.title);
    const domCount = parseDomUnreadCount();

    // Title count is primary when set; DOM badge serves as fallback
    const unreadCount = Math.max(titleCount, domCount);

    if (unreadCount !== lastReportedCount) {
      lastReportedCount = unreadCount;
      if (
        window.webkit &&
        window.webkit.messageHandlers &&
        window.webkit.messageHandlers.wafy
      ) {
        window.webkit.messageHandlers.wafy.postMessage(
          JSON.stringify({
            type: 'unread_count',
            count: unreadCount
          })
        );
      }
    }
  }

  /**
   * Debounced unread count evaluation to avoid excessive IPC overhead.
   */
  function scheduleUpdate() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      updateUnreadCount();
    }, DEBOUNCE_DELAY_MS);
  }

  // =========================================================================
  // 3. Observers Setup
  // =========================================================================

  // Observe <title> tag mutations
  function initTitleObserver() {
    const titleEl = document.querySelector('title');
    if (titleEl) {
      const titleObserver = new MutationObserver(() => {
        scheduleUpdate();
      });
      titleObserver.observe(titleEl, {
        childList: true,
        subtree: true,
        characterData: true
      });
    } else {
      // Retry once head/title is created
      const headObserver = new MutationObserver(() => {
        const title = document.querySelector('title');
        if (title) {
          headObserver.disconnect();
          initTitleObserver();
          scheduleUpdate();
        }
      });
      headObserver.observe(document.documentElement || document, {
        childList: true,
        subtree: true
      });
    }
  }

  // Observe DOM modifications in the app container
  function initDomObserver() {
    const target = document.body || document.documentElement;
    if (!target) {
      window.addEventListener('DOMContentLoaded', initDomObserver);
      return;
    }

    const domObserver = new MutationObserver(() => {
      scheduleUpdate();
    });

    domObserver.observe(target, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['aria-label', 'class', 'data-testid']
    });
  }

  // Periodic heartbeat fallback (every 3 seconds)
  setInterval(() => {
    scheduleUpdate();
  }, 3000);

  // Initialize observers and triggers
  initTitleObserver();
  initDomObserver();
  scheduleUpdate();

  // Also listen for standard page events
  window.addEventListener('load', scheduleUpdate);
  document.addEventListener('visibilitychange', scheduleUpdate);

  console.log('[Wafy] Badge & notification observers initialized.');
})();
