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
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  const page = await context.newPage();
  const errors = [];
  const refused = [];
  const dialogs = [];
  const external = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  // The refusals the script provokes on purpose show up as console errors with their status; count them
  // apart. A policy violation ("Refused to ...") is always a failure.
  page.on("console", (m) => {
    if (m.type() !== "error") { return; }
    if (m.text().includes("status of 403")) { refused.push(m.text()); } else { errors.push(m.text()); }
  });
  page.on("dialog", async (d) => { dialogs.push(d.message()); await d.dismiss(); });
  const origin = new URL(process.env.WEB_LINK).origin;
  page.on("request", (r) => { if (!r.url().startsWith(origin)) { external.push(r.url()); } });
  const shots = process.env.WEB_SHOTS;
  const shot = async (name) => { if (shots) { await page.waitForTimeout(900); await page.screenshot({ path: shots + "/" + name + ".png", fullPage: true }); } };
  const heading = async () => (await page.locator("main h2").first().textContent()).trim();

  // The first visit trades the link for a cookie and lands on a plain address.
  await page.goto(process.env.WEB_LINK);
  await page.waitForSelector("main h2");
  check(new URL(page.url()).search === "", "the token is gone from the address after the first visit");
  check((await heading()) === "Overview", "the overview opens");
  const session = (await context.cookies()).find((c) => c.name === "skillmirror_session");
  check(session && session.httpOnly && session.sameSite === "Strict", "the session cookie is HttpOnly and SameSite=Strict");
  check(!(await page.evaluate(() => document.cookie)).includes("skillmirror_session"), "a script cannot read the cookie");
  check((await page.textContent(".mode")).includes("Changes allowed"), "the sidebar says changes are allowed");
  await page.waitForSelector(".project");
  check((await page.$$eval(".project", (r) => r.length)) === 4, "four projects are listed");
  await page.waitForFunction(() => document.querySelector(".stat-value")?.textContent === "4");
  check(true, "the first counter counts up to the four projects");
  check((await page.content()).includes("&lt;img src=x onerror=alert(1)&gt;"), "a project named like markup is shown as text");
  check((await page.$$("main img")).length === 0 && dialogs.length === 0, "that name created no element and ran no script");
  await shot("overview");

  // The link works once.
  const second = await context.request.get(process.env.WEB_LINK, { maxRedirects: 0 });
  check(second.status() === 403, "the link does not work a second time");

  // A project opens and sends you to the sync page with it chosen.
  await page.click(".project >> nth=1 >> .project-head");
  await page.waitForSelector(".project-body");
  check((await page.textContent(".project-body")).includes("outdated"), "a project opens with what is outdated in it");
  await page.click("text=Sync this project");
  await page.waitForSelector("select");
  check(new URL(page.url()).pathname === "/sync" && new URL(page.url()).searchParams.has("project"), "the button goes to sync with the project in the address");
  check((await page.inputValue("select")) !== "", "and the project is chosen");

  // Sync: filter, pick, plan, apply.
  await page.waitForSelector("table.skills tbody tr");
  check((await page.$$eval("table.skills tbody tr", (r) => r.length)) === 2, "sync lists the two skills a project has and the vault knows");
  await page.selectOption("select", "");
  await page.fill("input[type=search]", "ast");
  check((await page.$$eval("table.skills tbody tr", (r) => r.length)) === 1, "the filter narrows the rows");
  await page.fill("input[type=search]", "");
  await page.click("text=None");
  await page.check("input[aria-label=coding]");
  await page.click("text=Plan…");
  await page.waitForSelector(".panel h3");
  check((await page.textContent(".panel h3")).includes("sync"), "a plan is shown first");
  check((await page.$$eval(".panel tbody tr", (r) => r.length)) === 2, "the plan has a row for each project that has coding");
  check(read("projects/one/.agents/skills/coding/SKILL.md").includes("v1"), "planning wrote nothing");
  await page.click(".panel .diff-file button >> nth=0");
  await page.waitForSelector("pre.diff");
  check((await page.$$eval("pre.diff .add", (l) => l.length)) >= 1 && (await page.$$eval("pre.diff .del", (l) => l.length)) >= 1, "the changes of a row open as a diff with added and removed lines marked");
  await shot("plan");
  await page.click("text=Apply this plan");
  await page.waitForSelector(".panel h3.ok");
  check((await page.textContent(".panel h3.ok")).includes("finished"), "the apply finishes");
  check(read("projects/one/.agents/skills/coding/SKILL.md").includes("v2"), "the skill was written in project one");
  check(read("projects/one/.agents/skills/astro/SKILL.md").includes("v1"), "the skill that was not ticked was left alone");
  await page.waitForFunction(() => document.querySelector("table.skills tbody tr")?.textContent.includes("up to date"));
  check(true, "the table above the result looks at the disk again, so coding is no longer outdated");
  await shot("applied");
  await page.click(".panel >> text=Done");

  // Push creates what is mandatory.
  await page.click("nav >> text=Push");
  await page.waitForSelector("table.skills tbody tr");
  await page.click("text=Plan…");
  await page.waitForSelector(".panel h3");
  await page.click("text=Apply this plan");
  await page.waitForSelector(".panel h3.ok");
  check(exists("projects/three/.agents/skills/tmux/SKILL.md"), "push installed the mandatory skill");

  // The skills page: a list and a card.
  await page.click("nav >> text=Skills");
  await page.waitForSelector(".card");
  check((await page.$$eval(".pick", (r) => r.length)) >= 4, "the skills page lists the vault's skills and the foreign folder");
  await page.click(".pick >> text=astro");
  check((await page.textContent(".card")).includes("Astro sites"), "the card shows the description of the picked skill");
  await page.fill("input[type=search]", "tmux");
  check((await page.$$eval(".pick", (r) => r.length)) === 1, "the filter narrows the list");
  await shot("skills");

  // History: undo the push.
  await page.click("nav >> text=History");
  await page.waitForSelector("table tbody tr");
  check((await page.$$eval("table tbody tr", (r) => r.length)) === 2, "history lists both runs");
  await page.click("text=Undo… >> nth=0");
  await page.waitForSelector(".panel h3");
  check(exists("projects/three/.agents/skills/tmux/SKILL.md"), "asking about an undo changes nothing");
  await shot("undo-question");
  await page.click("text=Undo this run");
  await page.waitForSelector(".panel h3.ok");
  check(!exists("projects/three/.agents/skills/tmux"), "undoing the push removed the skill it created");

  // Doctor and settings render.
  await page.click("nav >> text=Doctor");
  await page.waitForSelector("main h2");
  check((await heading()) === "Doctor", "the doctor page opens");
  await page.click("nav >> text=Settings");
  await page.waitForSelector(".settings");
  check((await page.textContent(".settings")).includes("flag"), "the settings say where the vault came from");

  // The theme button switches and the choice survives a reload.
  await page.click("text=Dark");
  check((await page.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "dark", "the theme button switches to dark");
  await shot("dark");
  await page.reload();
  await page.waitForSelector("main h2");
  check((await page.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "dark", "the choice is still there after a reload");

  // A reload on a deep address lands on the same view.
  await page.goto(origin + "/history");
  await page.waitForSelector("main h2");
  check((await heading()) === "History", "a direct visit of /history shows the history");

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
  check(errors.length === 0, "no console errors, no policy violations (" + errors.join(" | ") + ")");
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
