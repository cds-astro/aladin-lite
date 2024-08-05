import {test, expect, LocatorScreenshotOptions} from '@playwright/test';

async function open(page, exampleName) {
    await page.goto(`http://localhost:5173/tests/${exampleName}.html`);
    await page.setViewportSize({height: 600, width: 800});
}

const screenshotOptions: LocatorScreenshotOptions = {
    type: "png"
};

const toMatchOptions = {
    maxDiffPixels: 10
};

const pageTimeout = 2000;

test("al-ui-on", async ({ page }) => {
    await open(page, "al-ui-on");
    await page.waitForTimeout(pageTimeout);
    expect(await page.locator('canvas').nth(1).screenshot(screenshotOptions)).toMatchSnapshot(toMatchOptions);
});

test("al-ui-off", async ({ page }) => {
    await open(page, "al-ui-off");
    await page.waitForTimeout(pageTimeout);
    expect(await page.locator('canvas').nth(1).screenshot(screenshotOptions)).toMatchSnapshot(toMatchOptions);
});
