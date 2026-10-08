// Drives the report in Chromium and fails (exit 1) when something does not work.
// Needs Playwright for node: set PLAYWRIGHT_NODE to its folder (default /opt/node-tools/node_modules/playwright)
// and PLAYWRIGHT_CHROMIUM to the browser binary when it is not found by Playwright itself.
const path = process.env.PLAYWRIGHT_NODE || "/opt/node-tools/node_modules/playwright";
const { chromium } = require(path);

function check(ok, what) {
  console.log((ok ? "ok   " : "FAIL ") + what);
  if (!ok) { process.exitCode = 1; }
}

(async () => {
  const options = { args: ["--no-sandbox"] };
  if (process.env.PLAYWRIGHT_CHROMIUM) { options.executablePath = process.env.PLAYWRIGHT_CHROMIUM; }
  const browser = await chromium.launch(options);
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const errors = [];
  const requests = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  page.on("console", (m) => { if (m.type() === "error") { errors.push(m.text()); } });
  page.on("request", (r) => { if (!r.url().startsWith("file:")) { requests.push(r.url()); } });
  await page.goto("file://" + process.env.REPORT_HTML);
  const shots = process.env.REPORT_SHOTS;
  if (shots) { await page.screenshot({ path: shots + "/light.png", fullPage: true }); }

  check((await page.$$eval("tr[data-row]", (r) => r.length)) === 4, "the matrix has a row for every skill and the foreign folder");
  await page.fill("#filter", "astro");
  check((await page.$$eval("tr[data-row]:not([hidden])", (r) => r.length)) === 1, "the filter narrows the matrix to one row");
  check((await page.$$eval("section.skill:not([hidden])", (r) => r.length)) === 1, "the filter narrows the cards to one");
  await page.fill("#filter", "zzzz");
  check(await page.$eval("#nomatch", (e) => !e.hidden), "a filter without a match says so");
  await page.fill("#filter", "");
  check((await page.$$eval("tr[data-row]:not([hidden])", (r) => r.length)) === 4, "clearing the filter brings every row back");

  await page.click("#theme");
  check((await page.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "dark", "the button switches to dark");
  const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  check(bg !== "rgb(251, 251, 250)", "the dark theme changes the background (" + bg + ")");
  if (shots) { await page.screenshot({ path: shots + "/dark.png", fullPage: true }); }

  const link = await page.$("td.outdated a");
  check(link !== null, "an outdated cell is a link");
  if (link) {
    await link.click();
    await page.waitForTimeout(100);
    check((await page.$$eval("details.diff[open]", (d) => d.length)) === 1, "clicking it opens the diff");
  }
  check(requests.length === 0, "the page asked for nothing outside the file (" + requests.join(", ") + ")");
  check(errors.length === 0, "no console errors (" + errors.join(" | ") + ")");
  await browser.close();
})();
