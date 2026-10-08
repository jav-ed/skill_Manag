(function () {
  var root = document.documentElement;
  var theme = document.getElementById("theme");
  theme.addEventListener("click", function () {
    var dark = getComputedStyle(root).getPropertyValue("--is-dark").trim() === "1";
    root.setAttribute("data-theme", dark ? "light" : "dark");
  });
  var query = document.getElementById("filter");
  var none = document.getElementById("nomatch");
  function apply() {
    var text = query.value.trim().toLowerCase();
    var rows = 0;
    document.querySelectorAll("[data-find]").forEach(function (el) {
      var hit = !text || el.getAttribute("data-find").indexOf(text) >= 0;
      el.hidden = !hit;
      if (hit && el.hasAttribute("data-row")) { rows += 1; }
    });
    none.hidden = !text || rows > 0;
  }
  query.addEventListener("input", apply);
  function reveal() {
    var el = document.getElementById(location.hash.slice(1));
    while (el) {
      if (el.tagName === "DETAILS") { el.open = true; }
      el = el.parentElement;
    }
  }
  window.addEventListener("hashchange", reveal);
  reveal();
})();
