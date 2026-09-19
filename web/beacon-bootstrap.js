// This classic script runs before the module graph. Errors in static imports
// happen before last-beacon.js can install its own boot error handler.
(function () {
  'use strict';
  var app = document.getElementById('app');
  var actions = document.getElementById('start-actions');
  if (!app || !actions) return;
  var stopped = false;
  var observer;
  var timer;

  function stop() {
    stopped = true;
    clearTimeout(timer);
    if (observer) observer.disconnect();
    window.removeEventListener('error', onError, true);
    document.removeEventListener('securitypolicyviolation', onPolicyViolation);
  }

  function fail(message) {
    if (stopped || app.getAttribute('data-screen') !== 'loading') return;
    stop();
    // Keep the real hosted link supplied by the release, including in a file
    // preview. Never infer a deployment URL from the filename or current host.
    var hosted = document.getElementById('launch-hosted');
    while (actions.firstChild) actions.removeChild(actions.firstChild);
    var box = document.createElement('div');
    box.className = 'error-box';
    box.setAttribute('role', 'alert');
    var status = document.createElement('p');
    status.id = 'loading-status';
    status.textContent = message;
    box.appendChild(status);
    if (hosted && hosted.getAttribute('href')) box.appendChild(hosted);
    var retry = document.createElement('button');
    retry.className = 'secondary';
    retry.textContent = 'Try again';
    retry.addEventListener('click', function () { window.location.reload(); });
    box.appendChild(retry);
    actions.appendChild(box);
    app.setAttribute('data-launch-status', 'blocked');
  }

  function onError(event) {
    if (event.target && event.target.tagName === 'SCRIPT') {
      fail('The game files could not load. Check your connection, then try again or open the game in your browser.');
    } else if (event.message) {
      fail('The game could not start in this viewer. Try again or open it in Safari, Chrome, or Firefox.');
    }
  }

  function onPolicyViolation(event) {
    if (/^script-src/.test(event.effectiveDirective || '')) {
      fail('This preview blocked the game. Open it in your browser to play.');
    }
  }

  function checkSupport() {
    if (stopped) return;
    var script = document.createElement('script');
    if (!('noModule' in script) || typeof WebAssembly !== 'object') {
      fail('This viewer cannot run the game. Open it in a current version of Safari, Chrome, or Firefox.');
    } else if (document.querySelector('script[type="importmap"]') &&
      (!window.HTMLScriptElement.supports || !window.HTMLScriptElement.supports('importmap'))) {
      fail('This viewer cannot open the downloaded game. Use the online game in Safari, Chrome, or Firefox.');
    }
  }

  window.addEventListener('error', onError, true);
  document.addEventListener('securitypolicyviolation', onPolicyViolation);
  // A stalled or rejected module may never reach the app's boot() function.
  timer = setTimeout(function () {
    fail('The game has not started. This preview may not support games, or the connection may have stalled. Try again or open it in your browser.');
  }, 20000);
  observer = new MutationObserver(function () {
    // The main module owns all messages once it starts or reports its own error.
    if (app.getAttribute('data-screen') !== 'loading' || !document.getElementById('loading-status')) stop();
  });
  observer.observe(app, { attributes: true, attributeFilter: ['data-screen'], childList: true, subtree: true });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', checkSupport, { once: true });
  else checkSupport();
}());
