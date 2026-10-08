// Runs before the page is drawn, so the saved colour theme shows from the first frame.
(function () {
  try {
    var saved = localStorage.getItem('skillmirror-theme');
    if (saved === 'light' || saved === 'dark') document.documentElement.setAttribute('data-theme', saved);
  } catch (_) { /* storage can be blocked; the system setting then decides */ }
})();
