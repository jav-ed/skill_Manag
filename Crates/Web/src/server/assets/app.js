'use strict';
// The page script of `skillmirror web`. It builds everything it shows with text nodes, never with
// markup, so nothing it receives can become a tag.
(function () {
  const $ = (selector, root) => (root || document).querySelector(selector);
  const $$ = (selector, root) => Array.from((root || document).querySelectorAll(selector));

  function el(tag, attributes, ...children) {
    const node = document.createElement(tag);
    for (const [name, value] of Object.entries(attributes || {})) {
      if (name === 'class') node.className = value;
      else node.setAttribute(name, value);
    }
    for (const child of children) {
      if (child === null || child === undefined) continue;
      node.append(child instanceof Node ? child : document.createTextNode(String(child)));
    }
    return node;
  }

  async function call(method, path, body) {
    const options = { method, credentials: 'same-origin', headers: {} };
    if (method === 'POST') {
      options.headers['Content-Type'] = 'application/json';
      options.headers['X-Skillmirror'] = '1';
      options.body = JSON.stringify(body || {});
    }
    const response = await fetch(path, options);
    let data = {};
    try { data = await response.json(); } catch (_) { /* an empty body */ }
    if (!response.ok) throw new Error(data.error || response.status + ' ' + response.statusText);
    return data;
  }

  const panel = () => $('#panel');
  function show(...children) {
    const box = el('div', { class: 'box' }, ...children);
    panel().replaceChildren(box);
    box.scrollIntoView({ block: 'nearest' });
    return box;
  }
  function fail(error) { show(el('p', { class: 'bad' }, error.message || String(error))); }

  function actionCell(action) { return el('td', { class: 'act-' + action }, action); }

  function table(headings, rows) {
    return el('table', {},
      el('thead', {}, el('tr', {}, ...headings.map((h) => el('th', {}, h)))),
      el('tbody', {}, ...rows));
  }

  // ---- filter ---------------------------------------------------------------------------------
  document.addEventListener('input', (event) => {
    const box = event.target.closest('[data-filter]');
    if (!box) return;
    const query = box.value.trim().toLowerCase();
    for (const row of $$(box.dataset.filter + ' tbody tr')) {
      row.hidden = query !== '' && !row.textContent.toLowerCase().includes(query);
    }
  });

  // ---- plan and apply -------------------------------------------------------------------------
  // The lines of a unified diff, each as its own element with a class for added and removed lines.
  function diffBlock(file) {
    const pre = el('pre', { class: 'diff' });
    for (const line of file.text.split('\n')) {
      let kind = '';
      if (line.startsWith('+') && !line.startsWith('+++')) kind = 'add';
      else if (line.startsWith('-') && !line.startsWith('---')) kind = 'del';
      else if (line.startsWith('@@')) kind = 'hunk';
      pre.append(el('span', { class: kind }, line + '\n'));
    }
    return pre;
  }

  function changesOf(row) {
    if (!row.files || row.files.length === 0) return null;
    const parts = row.files.map((file) => el('div', {},
      el('strong', {}, file.path), ' ',
      el('span', { class: 'muted' }, file.kind + (file.added || file.removed ? ' +' + file.added + ' −' + file.removed : '')),
      file.note ? el('div', { class: 'muted' }, file.note) : null,
      file.text ? diffBlock(file) : null));
    return el('details', {}, el('summary', {}, 'changes (' + row.files.length + ' file' + (row.files.length === 1 ? '' : 's') + ')'), ...parts);
  }

  function renderPlan(plan) {
    const rows = plan.rows.map((row) => el('tr', {},
      el('td', { class: 'path' }, row.project), el('td', {}, row.skill),
      actionCell(row.action), el('td', { class: 'muted' }, row.detail, changesOf(row))));
    const changes = plan.create + plan.update;
    const summary = changes === 0
      ? 'Nothing to write.'
      : plan.create + ' to create, ' + plan.update + ' to update, in ' + plan.projects + ' project(s).'
        + (plan.failed ? ' ' + plan.failed + ' cannot be applied.' : '');
    const apply = el('button', { class: 'primary', 'data-action': 'apply', 'data-plan': plan.plan }, 'Apply this plan');
    if (changes === 0) apply.disabled = true;
    show(el('h3', {}, 'Plan: ' + plan.kind), el('p', {}, summary),
      table(['Project', 'Skill', 'Action', 'Detail'], rows),
      el('p', {}, apply, ' ', el('button', { 'data-action': 'cancel' }, 'Cancel')),
      el('p', { class: 'muted' }, 'Replaced folders are saved first; History can undo the run.'));
  }

  function renderUndo(plan) {
    const rows = plan.lines.map((line) => el('tr', {},
      el('td', { class: 'path' }, line.project), el('td', {}, line.skill),
      el('td', { class: 'muted' }, 'was ' + line.was), actionCell(line.step),
      el('td', { class: 'muted' }, line.message || '')));
    const apply = el('button', { class: 'danger', 'data-action': 'apply', 'data-plan': plan.plan }, 'Undo this run');
    if (plan.actionable === 0) apply.disabled = true;
    show(el('h3', {}, 'Undo ' + plan.command + ' of ' + plan.date),
      el('p', {}, plan.actionable + ' folder(s) can be put right' + (plan.failed ? ', ' + plan.failed + ' cannot.' : '.')),
      table(['Project', 'Skill', 'Run did', 'Undo', 'Detail'], rows),
      el('p', {}, apply, ' ', el('button', { 'data-action': 'cancel' }, 'Cancel')));
  }

  function renderFinished(done) {
    const rows = done.lines.map((line) => el('tr', {},
      el('td', { class: 'path' }, line.project), el('td', {}, line.skill),
      actionCell(line.outcome), el('td', { class: 'muted' }, line.detail || '')));
    const note = [];
    if (done.backup) note.push(el('p', { class: 'muted' }, 'Backup run ' + done.backup + ': History can undo it.'));
    for (const warning of done.warnings) note.push(el('p', { class: 'warn' }, warning));
    show(el('h3', { class: done.failed ? 'bad' : 'ok' }, done.title + (done.failed ? ' with ' + done.failed + ' failure(s)' : '')),
      table(['Project', 'Skill', 'Result', 'Detail'], rows), ...note,
      el('p', {}, el('button', { 'data-action': 'reload' }, 'Reload the page')));
  }

  async function watch(job) {
    const bar = el('progress', { max: '1', value: '0' });
    show(el('h3', {}, 'Working…'), bar);
    for (;;) {
      const view = await call('GET', '/api/job/' + encodeURIComponent(job));
      if (view.state === 'running') {
        bar.max = Math.max(view.total, 1);
        bar.value = view.done;
        await new Promise((resolve) => setTimeout(resolve, 250));
      } else if (view.state === 'done') {
        return renderFinished(view);
      } else {
        throw new Error(view.message);
      }
    }
  }

  const handlers = {
    async rescan() { await call('POST', '/api/rescan'); location.reload(); },
    reload() { location.reload(); },
    cancel() { panel().replaceChildren(); },
    'select-changed'() { for (const row of $$('#skills tbody tr')) { const box = $('input[type=checkbox]', row); if (box) box.checked = row.dataset.changes !== '0' && !row.hidden; } },
    'select-all'() { for (const row of $$('#skills tbody tr')) { const box = $('input[type=checkbox]', row); if (box) box.checked = !row.hidden; } },
    'select-none'() { for (const box of $$('#skills input[type=checkbox]')) box.checked = false; },
    async plan() {
      const skills = $$('#skills input[type=checkbox]:checked').map((box) => box.value);
      if (skills.length === 0) return show(el('p', { class: 'warn' }, 'Tick at least one skill.'));
      const project = $('#project') ? $('#project').value : '';
      renderPlan(await call('POST', '/api/plan', { kind: $('#skills').dataset.kind, skills, project: project || null }));
    },
    async apply(button) {
      button.disabled = true;
      const started = await call('POST', '/api/apply', { plan: button.dataset.plan });
      await watch(started.job);
    },
    async undo(button) {
      renderUndo(await call('POST', '/api/undo-plan', { run: button.dataset.run }));
    },
  };

  document.addEventListener('click', (event) => {
    const button = event.target.closest('[data-action]');
    if (!button || !handlers[button.dataset.action]) return;
    Promise.resolve(handlers[button.dataset.action](button)).catch(fail);
  });
})();
