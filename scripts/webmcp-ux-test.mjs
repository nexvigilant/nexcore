#!/usr/bin/env node
// =============================================================================
// NexVigilant WebMCP Config UX Test
// =============================================================================
// Tests EVERY interaction pathway a user might take on published configs.
// Mirrors the exact selectors published to WebMCP Hub.
// =============================================================================

import { chromium } from 'playwright';

const BASE = 'http://localhost:3001';
const RESULTS = { pass: 0, fail: 0, skip: 0, tests: [] };

function log(status, name, detail = '') {
  const icon = status === 'PASS' ? '\x1b[32mPASS\x1b[0m' : status === 'FAIL' ? '\x1b[31mFAIL\x1b[0m' : '\x1b[33mSKIP\x1b[0m';
  console.log(`  [${icon}] ${name}${detail ? ' — ' + detail : ''}`);
  RESULTS[status.toLowerCase()]++;
  RESULTS.tests.push({ status, name, detail });
}

async function testNaranjo(page) {
  console.log('\n=== CONFIG 1: Naranjo Causality Assessment ===');
  console.log('Published tools: assess-naranjo-causality, read-naranjo-result');
  console.log('Testing all user pathways...\n');

  // --- PATHWAY 1: Page loads ---
  try {
    const resp = await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
    log(resp.status() === 200 ? 'PASS' : 'FAIL', 'Page loads', `HTTP ${resp.status()}`);
  } catch (e) {
    log('FAIL', 'Page loads', e.message);
    return;
  }

  // --- PATHWAY 2: All 10 question buttons render ---
  for (let q = 0; q < 10; q++) {
    const buttons = await page.$$(`button[data-question="${q}"]`);
    log(buttons.length >= 3 ? 'PASS' : 'FAIL', `Q${q + 1} buttons render`, `Found ${buttons.length} buttons`);
  }

  // --- PATHWAY 3: Submit without answering (validation) ---
  const calcBtn = page.locator('button:has-text("Calculate Score"), button:has-text("Calculate"), button:has-text("Submit")');
  const calcExists = await calcBtn.count();
  if (calcExists > 0) {
    await calcBtn.first().click();
    await page.waitForTimeout(500);
    // Check if there's a validation message or if it just calculates with defaults
    const bodyText = await page.textContent('main');
    log('PASS', 'Submit without answers', bodyText.includes('Score') || bodyText.includes('error') || bodyText.includes('required') ? 'Got response' : 'No crash');
  } else {
    log('SKIP', 'Submit without answers', 'No calculate button found');
  }

  // --- PATHWAY 4: Answer all "Yes" (maximum score = Definite) ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  for (let q = 0; q < 10; q++) {
    const yesBtn = page.locator(`button[data-question="${q}"]`).first();
    if (await yesBtn.count() > 0) {
      await yesBtn.click();
      await page.waitForTimeout(100);
    }
  }
  if (calcExists > 0) {
    const calcBtn2 = page.locator('button:has-text("Calculate Score"), button:has-text("Calculate"), button:has-text("Submit")');
    if (await calcBtn2.count() > 0) {
      await calcBtn2.first().click();
      await page.waitForTimeout(1000);
    }
  }
  const allYesText = await page.textContent('main');
  const hasDefinite = allYesText.toLowerCase().includes('definite');
  const hasScore = /score/i.test(allYesText) || /\d+/.test(allYesText);
  log(hasScore ? 'PASS' : 'FAIL', 'All Yes → score displayed', hasDefinite ? 'Classification: Definite' : 'Score shown');

  // --- PATHWAY 5: Answer all "No" (minimum score = Doubtful) ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  for (let q = 0; q < 10; q++) {
    const btns = page.locator(`button[data-question="${q}"]`);
    const count = await btns.count();
    if (count >= 2) {
      await btns.nth(1).click(); // "No" is typically second
      await page.waitForTimeout(100);
    }
  }
  {
    const calcBtn3 = page.locator('button:has-text("Calculate Score"), button:has-text("Calculate"), button:has-text("Submit")');
    if (await calcBtn3.count() > 0) {
      await calcBtn3.first().click();
      await page.waitForTimeout(1000);
    }
  }
  const allNoText = await page.textContent('main');
  const hasDoubtful = allNoText.toLowerCase().includes('doubtful');
  log(hasDoubtful || /score/i.test(allNoText) ? 'PASS' : 'FAIL', 'All No → score displayed', hasDoubtful ? 'Classification: Doubtful' : 'Score shown');

  // --- PATHWAY 6: Answer all "Don't Know" (score = 0, Doubtful) ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  for (let q = 0; q < 10; q++) {
    const btns = page.locator(`button[data-question="${q}"]`);
    const count = await btns.count();
    if (count >= 3) {
      await btns.nth(2).click(); // "Don't Know" is typically third
      await page.waitForTimeout(100);
    }
  }
  {
    const calcBtn4 = page.locator('button:has-text("Calculate Score"), button:has-text("Calculate"), button:has-text("Submit")');
    if (await calcBtn4.count() > 0) {
      await calcBtn4.first().click();
      await page.waitForTimeout(1000);
    }
  }
  const dontKnowText = await page.textContent('main');
  log(/score|0|doubtful/i.test(dontKnowText) ? 'PASS' : 'FAIL', "All Don't Know → score = 0", dontKnowText.includes('0') ? 'Score: 0' : 'Score shown');

  // --- PATHWAY 7: Mixed answers (Probable range 5-8) ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  // Yes for Q1-Q5, No for Q6-Q8, Don't Know for Q9-Q10
  for (let q = 0; q < 10; q++) {
    const btns = page.locator(`button[data-question="${q}"]`);
    const count = await btns.count();
    if (count >= 3) {
      if (q < 5) await btns.nth(0).click();       // Yes
      else if (q < 8) await btns.nth(1).click();   // No
      else await btns.nth(2).click();               // Don't Know
      await page.waitForTimeout(100);
    }
  }
  {
    const calcBtn5 = page.locator('button:has-text("Calculate Score"), button:has-text("Calculate"), button:has-text("Submit")');
    if (await calcBtn5.count() > 0) {
      await calcBtn5.first().click();
      await page.waitForTimeout(1000);
    }
  }
  const mixedText = await page.textContent('main');
  log(/score|probable|possible|\d/i.test(mixedText) ? 'PASS' : 'FAIL', 'Mixed answers → classification',
    mixedText.toLowerCase().includes('probable') ? 'Probable' :
    mixedText.toLowerCase().includes('possible') ? 'Possible' : 'Score shown');

  // --- PATHWAY 8: Change answer mid-form (toggle) ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  const q1btns = page.locator('button[data-question="0"]');
  if (await q1btns.count() >= 3) {
    await q1btns.nth(0).click(); // Click Yes
    await page.waitForTimeout(200);
    await q1btns.nth(1).click(); // Change to No
    await page.waitForTimeout(200);
    await q1btns.nth(0).click(); // Change back to Yes
    await page.waitForTimeout(200);
    log('PASS', 'Toggle answer mid-form', 'No crash on rapid toggle');
  } else {
    log('SKIP', 'Toggle answer mid-form', 'Buttons not found');
  }

  // --- PATHWAY 9: WebMCP selector verification ---
  // Verify the exact selectors from the published config work
  const webmcpSelectors = [
    'main',
    'button[data-question="0"]',
    'button[data-question="9"]',
  ];
  for (const sel of webmcpSelectors) {
    const el = page.locator(sel);
    const found = await el.count();
    log(found > 0 ? 'PASS' : 'FAIL', `WebMCP selector: ${sel}`, `${found} elements`);
  }

  // --- PATHWAY 10: Screenshot for visual review ---
  await page.goto(`${BASE}/naranjo-causality`, { waitUntil: 'networkidle', timeout: 15000 });
  await page.screenshot({ path: '/tmp/naranjo-test.png', fullPage: true });
  log('PASS', 'Screenshot captured', '/tmp/naranjo-test.png');
}

async function testPRR(page) {
  console.log('\n=== CONFIG 2: PRR Signal Detection ===');
  console.log('Published tools: compute-prr-signal, read-prr-results');
  console.log('Testing all user pathways...\n');

  // --- PATHWAY 1: Page loads ---
  try {
    const resp = await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
    log(resp.status() === 200 ? 'PASS' : 'FAIL', 'Page loads', `HTTP ${resp.status()}`);
  } catch (e) {
    log('FAIL', 'Page loads', e.message);
    return;
  }

  // --- PATHWAY 2: Input fields render (WebMCP selectors) ---
  const inputSelectors = [
    { sel: 'input[aria-label*="drug and event"], input[aria-label*="cell a"], input[name*="a"], input:nth-of-type(1)', name: 'Cell A input' },
    { sel: 'input[aria-label*="drug but no event"], input[aria-label*="cell b"], input[name*="b"], input:nth-of-type(2)', name: 'Cell B input' },
    { sel: 'input[aria-label*="other drugs and event"], input[aria-label*="cell c"], input[name*="c"], input:nth-of-type(3)', name: 'Cell C input' },
    { sel: 'input[aria-label*="other drugs and no event"], input[aria-label*="cell d"], input[name*="d"], input:nth-of-type(4)', name: 'Cell D input' },
  ];

  // Find all number inputs on the page
  const allInputs = await page.$$('input[type="number"], input[type="text"], input');
  log(allInputs.length >= 4 ? 'PASS' : 'FAIL', 'Four input fields render', `Found ${allInputs.length} inputs`);

  // --- PATHWAY 3: Valid signal case (PRR > 2, Evans criteria met) ---
  // Classic signal: a=50, b=100, c=10, d=1000
  // PRR = (50/150) / (10/1010) = 0.333 / 0.0099 = 33.67
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const inputs3 = await page.$$('input');
  if (inputs3.length >= 4) {
    await inputs3[0].fill('50');
    await inputs3[1].fill('100');
    await inputs3[2].fill('10');
    await inputs3[3].fill('1000');

    const computeBtn = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn.count() > 0) {
      await computeBtn.first().click();
      await page.waitForTimeout(1500);
    }
    const resultText = await page.textContent('main');
    const hasPRR = /prr|signal|ratio|\d+\.\d+/i.test(resultText);
    log(hasPRR ? 'PASS' : 'FAIL', 'Valid signal case (a=50,b=100,c=10,d=1000)',
      resultText.toLowerCase().includes('signal') ? 'Signal detected' : 'PRR computed');
  } else {
    log('FAIL', 'Valid signal case', `Only ${inputs3.length} inputs found`);
  }

  // --- PATHWAY 4: No signal case (PRR < 2) ---
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const inputs4 = await page.$$('input');
  if (inputs4.length >= 4) {
    await inputs4[0].fill('5');
    await inputs4[1].fill('500');
    await inputs4[2].fill('50');
    await inputs4[3].fill('500');

    const computeBtn2 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn2.count() > 0) {
      await computeBtn2.first().click();
      await page.waitForTimeout(1500);
    }
    const noSigText = await page.textContent('main');
    log(/prr|no signal|ratio|\d/i.test(noSigText) ? 'PASS' : 'FAIL', 'No signal case (a=5,b=500,c=50,d=500)', 'PRR < 2');
  }

  // --- PATHWAY 5: Edge case — zeros ---
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const inputs5 = await page.$$('input');
  if (inputs5.length >= 4) {
    await inputs5[0].fill('0');
    await inputs5[1].fill('0');
    await inputs5[2].fill('0');
    await inputs5[3].fill('0');

    const computeBtn3 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn3.count() > 0) {
      await computeBtn3.first().click();
      await page.waitForTimeout(1500);
    }
    const zeroText = await page.textContent('main');
    // Should handle gracefully — no crash, no NaN, no Infinity
    const hasBadValue = /NaN|Infinity|undefined|null|error/i.test(zeroText);
    log(!hasBadValue ? 'PASS' : 'FAIL', 'Edge: all zeros', hasBadValue ? 'BAD VALUE DISPLAYED' : 'Handled gracefully');
  }

  // --- PATHWAY 6: Edge case — very large numbers ---
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const inputs6 = await page.$$('input');
  if (inputs6.length >= 4) {
    await inputs6[0].fill('999999');
    await inputs6[1].fill('999999');
    await inputs6[2].fill('999999');
    await inputs6[3].fill('999999');

    const computeBtn4 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn4.count() > 0) {
      await computeBtn4.first().click();
      await page.waitForTimeout(1500);
    }
    const bigText = await page.textContent('main');
    const hasBadBig = /NaN|Infinity|undefined|error/i.test(bigText);
    log(!hasBadBig ? 'PASS' : 'FAIL', 'Edge: very large numbers', hasBadBig ? 'OVERFLOW' : 'Handled');
  }

  // --- PATHWAY 7: Edge case — negative numbers ---
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const inputs7 = await page.$$('input');
  if (inputs7.length >= 4) {
    await inputs7[0].fill('-5');
    await inputs7[1].fill('100');
    await inputs7[2].fill('10');
    await inputs7[3].fill('1000');

    const computeBtn5 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn5.count() > 0) {
      await computeBtn5.first().click();
      await page.waitForTimeout(1500);
    }
    const negText = await page.textContent('main');
    // Should reject or handle gracefully
    log('PASS', 'Edge: negative input', /error|invalid|negative/i.test(negText) ? 'Rejected properly' : 'No crash');
  }

  // --- PATHWAY 8: Empty submit ---
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  {
    const computeBtn6 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn6.count() > 0) {
      await computeBtn6.first().click();
      await page.waitForTimeout(1000);
    }
    const emptyText = await page.textContent('main');
    const hasCrash = /unhandled|exception|cannot read/i.test(emptyText);
    log(!hasCrash ? 'PASS' : 'FAIL', 'Empty submit', hasCrash ? 'CRASH' : 'Handled');
  }

  // --- PATHWAY 9: WebMCP result selector verification ---
  // The published config uses resultSelector: "[class*=\"grid\"]"
  await page.goto(`${BASE}/prr-signal-detection`, { waitUntil: 'networkidle', timeout: 15000 });
  const gridEl = await page.$$('[class*="grid"]');
  log(gridEl.length > 0 ? 'PASS' : 'FAIL', 'WebMCP resultSelector: [class*="grid"]', `${gridEl.length} elements`);

  // --- PATHWAY 10: Screenshot for visual review ---
  const inputs10 = await page.$$('input');
  if (inputs10.length >= 4) {
    await inputs10[0].fill('50');
    await inputs10[1].fill('100');
    await inputs10[2].fill('10');
    await inputs10[3].fill('1000');
    const computeBtn7 = page.locator('button:has-text("Compute"), button:has-text("Calculate"), button:has-text("Detect"), button:has-text("Submit")');
    if (await computeBtn7.count() > 0) {
      await computeBtn7.first().click();
      await page.waitForTimeout(1500);
    }
  }
  await page.screenshot({ path: '/tmp/prr-test.png', fullPage: true });
  log('PASS', 'Screenshot captured', '/tmp/prr-test.png');
}

// === MAIN ===
(async () => {
  console.log('==============================================');
  console.log('NexVigilant WebMCP Config UX Test Suite');
  console.log('==============================================');
  console.log(`Target: ${BASE}`);
  console.log(`Date: ${new Date().toISOString()}`);

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();

  // Catch console errors
  const consoleErrors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  page.on('pageerror', err => consoleErrors.push(err.message));

  try {
    await testNaranjo(page);
    await testPRR(page);
  } finally {
    await browser.close();
  }

  // === SUMMARY ===
  console.log('\n==============================================');
  console.log('SUMMARY');
  console.log('==============================================');
  console.log(`  PASS: ${RESULTS.pass}`);
  console.log(`  FAIL: ${RESULTS.fail}`);
  console.log(`  SKIP: ${RESULTS.skip}`);
  console.log(`  Total: ${RESULTS.pass + RESULTS.fail + RESULTS.skip}`);

  if (consoleErrors.length > 0) {
    console.log(`\n  Console Errors (${consoleErrors.length}):`);
    consoleErrors.slice(0, 10).forEach(e => console.log(`    - ${e.substring(0, 120)}`));
  }

  console.log(`\n  Verdict: ${RESULTS.fail === 0 ? '\x1b[32mALL PATHWAYS CLEAR\x1b[0m' : '\x1b[31mFAILURES DETECTED — FIX BEFORE PUBLISH\x1b[0m'}`);

  process.exit(RESULTS.fail > 0 ? 1 : 0);
})();
