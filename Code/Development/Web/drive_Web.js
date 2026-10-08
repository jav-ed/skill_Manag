// Drives `skillmirror web --allow-write` in Chromium and fails (exit 1) when something does not work.
// Needs Playwright for node: set PLAYWRIGHT_NODE to its folder (default /opt/node-tools/node_modules/playwright)
// and PLAYWRIGHT_CHROMIUM to the browser binary when it is not found by Playwright itself.
const fs = require("fs");
const path = process.env.PLAYWRIGHT_NODE || "/opt/node-tools/node_modules/playwright";
const { chromium } = require(path);

const work = process.env.WEB_WORK;
const read = (rel) => fs.readFileSync(work + "/" + rel, "utf8");
const exists = (rel) => fs.existsSync(work + "/" + rel);

function check(ok, what) {
  console.log((ok ? "ok   " : "FAIL ") + what);
  if (!ok) { process.exitCode = 1; }
}

(async () => {
  const options = { args: ["--no-sandbox"] };
  if (process.env.PLAYWRIGHT_CHROMIUM) { options.executablePath = process.env.PLAYWRIGHT_CHROMIUM; }
  const browser = await chromium.launch(options);
  const context = await browser.newContext({ viewport: { width: 1200, height: 900 } });
  const page = await context.newPage();
  const errors = [];
  const dialogs = [];
  const external = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  // The refusals the script provokes on purpose show up as console errors with their status; count them apart.
  const refused = [];
  page.on("console", (m) => {
    if (m.type() !== "error") { return; }
    if (m.text().includes("status of 403")) { refused.push(m.text()); } else { errors.push(m.text()); }
  });
  page.on("dialog", async (d) => { dialogs.push(d.message()); await d.dismiss(); });
  const origin = new URL(process.env.WEB_LINK).origin;
  page.on("request", (r) => { if (!r.url().startsWith(origin)) { external.push(r.url()); } });
  const shots = process.env.WEB_SHOTS;
  const shot = async (name) => { if (shots) { await page.screenshot({ path: shots + "/" + name + ".png", fullPage: true }); } };

  // The first visit trades the link for a cookie and lands on a plain address.
  await page.goto(process.env.WEB_LINK);
  check(new URL(page.url()).search === "", "the token is gone from the address after the first visit");
  check((await page.textContent("h2")) === "Overview", "the overview opens");
  const cookies = await context.cookies();
  const session = cookies.find((c) => c.name === "skillmirror_session");
  check(session && session.httpOnly && session.sameSite === "Strict", "the session cookie is HttpOnly and SameSite=Strict");
  check(!(await page.evaluate(() => document.cookie)).includes("skillmirror_session"), "a script cannot read the cookie");
  check((await page.textContent(".mode")).includes("changes allowed"), "the header says changes are allowed");
  check((await page.$$eval("tbody tr", (r) => r.length)) === 4, "four projects are listed");
  check((await page.content()).includes("&lt;img src=x onerror=alert(1)&gt;"), "a project named like markup is shown as text");
  check((await page.$$("img")).length === 0 && dialogs.length === 0, "that name created no element and ran no script");
  await shot("overview");

  // The link works once.
  const second = await context.request.get(process.env.WEB_LINK, { maxRedirects: 0 });
  check(second.status() === 403, "the link does not work a second time");

  // Sync: filter, plan, apply.
  await page.click("nav a[href='/sync']");
  check((await page.$$eval("#skills tbody tr", (r) => r.length)) === 2, "sync lists the two skills a project has and the vault knows");
  await page.fill("[data-filter]", "ast");
  check((await page.$$eval("#skills tbody tr:not([hidden])", (r) => r.length)) === 1, "the filter narrows the rows");
  await page.fill("[data-filter]", "");
  await page.click("[data-action=select-none]");
  await page.check("#skills input[value=coding]");
  await page.click("[data-action=plan]");
  await page.waitForSelector("#panel table");
  check((await page.textContent("#panel h3")).includes("sync"), "a plan is shown first");
  await page.click("#panel details summary >> nth=0");
  check((await page.$$eval("#panel pre.diff .add", (l) => l.length)) >= 1, "the changes of a row open as a diff with added lines marked");
  check((await page.$$eval("#panel pre.diff .del", (l) => l.length)) >= 1, "and removed lines");
  check((await page.$$eval("#panel tbody tr", (r) => r.length)) === 2, "the plan has a row for each project that has coding");
  check(read("projects/one/.agents/skills/coding/SKILL.md").includes("v1"), "planning wrote nothing");
  await shot("plan");
  await page.click("[data-action=apply]");
  await page.waitForSelector("#panel h3.ok");
  check((await page.textContent("#panel h3")).includes("finished"), "the apply finishes");
  check(read("projects/one/.agents/skills/coding/SKILL.md").includes("v2"), "the skill was written in project one");
  check(read("projects/one/.agents/skills/astro/SKILL.md").includes("v1"), "the skill that was not ticked was left alone");
  await shot("applied");

  // Push creates what is mandatory.
  await page.click("nav a[href='/push']");
  await page.click("[data-action=plan]");
  await page.waitForSelector("#panel table");
  await page.click("[data-action=apply]");
  await page.waitForSelector("#panel h3.ok");
  check(exists("projects/three/.agents/skills/tmux/SKILL.md"), "push installed the mandatory skill");

  // History: undo the push and the sync.
  await page.click("nav a[href='/history']");
  check((await page.$$eval("[data-action=undo]", (b) => b.length)) === 2, "history lists both runs");
  await page.click("[data-action=undo] >> nth=0");
  await page.waitForSelector("#panel button[data-action=apply]");
  check(exists("projects/three/.agents/skills/tmux/SKILL.md"), "asking about an undo changes nothing");
  await shot("undo-question");
  await page.click("#panel button[data-action=apply]");
  await page.waitForSelector("#panel h3.ok");
  check(!exists("projects/three/.agents/skills/tmux"), "undoing the push removed the skill it created");

  // The refusals, from inside the page.
  const refusals = await page.evaluate(async () => {
    const send = (headers) => fetch("/api/plan", { method: "POST", headers, body: JSON.stringify({ kind: "sync", skills: ["coding"] }) }).then((r) => r.status);
    return {
      noMark: await send({ "Content-Type": "application/json" }),
      form: await send({ "Content-Type": "application/x-www-form-urlencoded", "X-Skillmirror": "1" }),
    };
  });
  check(refusals.noMark === 403 && refusals.form === 403, "a change without our header or as a form is refused");
  const stranger = await browser.newContext();
  const bare = await stranger.request.get(origin + "/");
  check(bare.status() === 401, "a browser without the session gets nothing");

  check(refused.length === 2, "the two refusals provoked on purpose were the only 403s (" + refused.length + ")");
  check(external.length === 0, "the pages asked for nothing outside the server (" + external.join(", ") + ")");
  check(errors.length === 0, "no console errors (" + errors.join(" | ") + ")");
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
